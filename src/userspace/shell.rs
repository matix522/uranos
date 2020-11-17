use crate::syscall::files::File;
use crate::{uprintln,uprint, euprintln};
use crate::syscall::{create_task, get_child_return_value, yield_cpu, get_pid, set_pipe_read_on_pid};
use alloc::string::String;
use alloc::vec::Vec;
#[derive(Debug)]
enum ParseError {
    UnknownProgram(String),
    QuoteUnclosed,
}

type ErrorCode = u32;

type Pid = u64;

const UTF8_ERROR: u32 = 20;

const READ_ERROR: u32 = 30;

pub(super) fn shell_impl(_args: &[&[u8]]) -> Result<(), ErrorCode> {
    let mut command_line = String::new();
    let stdin: File = File::get_stdin();

    let mut buffer = Vec::<u8>::new();
    buffer.resize(4096, 0);

    let buffer = &mut buffer[..];
    print_prompt();
    'main_loop: loop {
        let read_count = stdin.read(4096, buffer).map_err(|_| READ_ERROR)?;
        if read_count == 0 {
            yield_cpu();
        }
        let read_bytes = &mut buffer[..read_count];

        let mut unparsed_string = core::str::from_utf8(read_bytes).map_err(|_| UTF8_ERROR)?;

        while let Some(line_end_pos) = unparsed_string.find('\n') {
            let (rest_of_line, new_line) = unparsed_string.split_at(line_end_pos + 1);
            unparsed_string = new_line;

            add_new_input(&mut command_line, rest_of_line);
            run_commands(command_line.trim());
            command_line.clear();
            print_prompt();
        }
        if unparsed_string.len() > 0 {
            add_new_input(&mut command_line, unparsed_string);
        }
    }
    Ok(())
}

fn add_new_input(command_line: &mut String, new_input: &str) {
    let mut new_input: String = new_input.into();

    let (finished, deletions) = process_input(&mut new_input);

    *command_line += &new_input;
    if !finished {
        let (finished, new_deletions) = process_input(command_line);
        if !finished {
            if let Some(index) = command_line.find(|c| c != '\u{7f}') {
                let (_, tail) = command_line.split_at(index);
                *command_line = tail.into();
            } else {
                *command_line = String::from("");
            }
        }
        if new_deletions > 0 {
            uprint!("\x1B[{}D\x1B[0K", new_deletions);
        }
        uprint!("{}", &new_input);
    } else {
        uprint!("{}", &new_input);
    }
}

/// returns pair of flag that is true if string was processed completly and count of removed characters
fn process_input(string: &mut String) -> (bool, usize) {
    let mut cleared = String::new();
    let mut deletions = 0;
    let mut finished = true;

    let mut skip = 0;
    for c in string.chars() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match c {
            '\u{7f}' => {
                if cleared.len() > 0 && !cleared.ends_with('\u{7f}') {
                    cleared.truncate(cleared.len() - 1);
                    deletions += 1;
                } else {
                    cleared.push(c);
                    finished = false;
                }
            }
            ' '..='~' | '\n' => cleared.push(c),
            '\u{1b}' => {
                skip = 2;
            }
            _ => {}
        }
    }
    core::mem::swap(string, &mut cleared);
    (finished, deletions)
}

trait ShellSplit {
    fn shell_split<Q, S>(&self, quotes: Q, spliters: S) -> Result<Vec<&Self>, ParseError>
    where
        Q: Iterator<Item = char> + Clone,
        S: Iterator<Item = char> + Clone;
}
impl ShellSplit for str {
    fn shell_split<Q, S>(&self, q: Q, s: S) -> Result<Vec<&Self>, ParseError>
    where
        Q: Iterator<Item = char> + Clone,
        S: Iterator<Item = char> + Clone,
    {
        let mut pieces = Vec::new();
        let mut in_quotes = false;
        let mut last_pipe = 0;

        for (i, c) in self.chars().enumerate() {
            let mut quotes = q.clone();
            let mut spliters = s.clone();
            if quotes.any(|quote| quote == c) {
                in_quotes = !in_quotes;
            } else if !in_quotes && spliters.any(|split| split == c) {
                pieces.push(self[last_pipe..i].trim());
                last_pipe = i + 1;
            }
        }
        if last_pipe != self.len() {
            pieces.push(self[last_pipe..].trim())
        }
        if in_quotes {
            return Err(ParseError::QuoteUnclosed);
        }
        Ok(pieces)
    }
}
fn run_commands(command_line: &str) -> Result<ErrorCode, ParseError> {
    let command_chain = command_line.shell_split("\'\"".chars(), ";".chars())?;
    for base_cmd in command_chain {
        let indivdual_comands = base_cmd.shell_split("\'\"".chars(), "|".chars())?;
        
        let my_pid = get_pid() as u64;
        let mut input_source_pid = my_pid ;

        for command in indivdual_comands {
            let pid = match run_command(command, input_source_pid) {
                Ok(pid) => pid,
                Err(parse_error) => {
                    euprintln!("Shell Error: {:?}", parse_error);
                    continue;
                },
            };
            input_source_pid = pid;
        }
        if input_source_pid != my_pid{
            set_pipe_read_on_pid(input_source_pid);
            let end =File::get_pipein();

            let mut buffer = Vec::<u8>::new();
            buffer.resize(4096, 0);
        
            let buffer = &mut buffer[..];
            
            while let Ok (read_count ) = end.read(4096, buffer) {
                let read_bytes = &mut buffer[..read_count];

                let mut unparsed_string = core::str::from_utf8(read_bytes).unwrap_or("ERROR\n");
                crate::uprint!("{}", unparsed_string);
            }

            let ret_val = await_child(input_source_pid);
            crate::euprintln!("Process Exited with code {}", ret_val);
        } 


    }
    Ok(0)
}
fn run_command(command: &str, input_source_pid : u64) -> Result<Pid, ParseError> {
    let words = command.shell_split("\'\"".chars(), " ".chars())?;
    let (head, tail) = words.split_at(1);
    let command_name = head[0];

    // let bytes: Vec<&[u8]> = tail.iter().map(|string| string.as_bytes()).collect();

    for &(name, function) in super::PROGRAMS.iter() {
        if name == command_name {
            let child_pid = create_task(function, tail, true, Some(input_source_pid));
            return Ok(child_pid);
        }
    }
    Err(ParseError::UnknownProgram(command_name.into()))
}

fn await_child(child_pid: u64) -> u32 {
    loop {
        if let Some(ret) = get_child_return_value(child_pid) {
            return ret;
        } else {
            yield_cpu();
        }
    }
}

fn print_prompt() {
    uprint!("\u{1FA90} default@uranos | \u{1F5C1}  / > ");
}
