use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

#[no_mangle]
#[inline(never)]
pub extern "C" fn _true(_argc: usize, _argv: *const &[u8]) -> u32 {
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _false(_argc: usize, _argv: *const &[u8]) -> u32 {
    1
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn simple_cat(argc: usize, argv: *const &[u8]) -> u32 {
    use crate::syscall::files::File;
    use crate::syscall::*;
    use core::convert::TryInto;
    use core::str::from_utf8;

    if argc != 1 && argc != 2 {
        print::print("Invalid number of arguments\n");
        return 1;
    }

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    let filename = match from_utf8(args[0]) {
        Ok(val) => val,
        Err(_) => {
            print::print("Expected valid utf8 string\n");
            return 2;
        }
    };

    let out_file = if argc == 2 {
        let bytes: &[u8; 8] = match args[1].try_into() {
            Ok(val) => val,
            Err(_) => {
                print::print("Invalid out type flag value");
                return 3;
            }
        };
        if u64::from_le_bytes(*bytes) > 0 {
            File::get_pipeout()
        } else {
            File::get_stdout()
        }
    } else {
        File::get_stdout()
    };

    let f = match File::open(filename, false) {
        Ok(f) => f,
        Err(e) => {
            print::print(&format!("A file error occured during open: {:?}\n", e));
            return 3;
        }
    };

    let mut buffer = [0u8; 64];
    loop {
        let count = match f.read(64, &mut buffer) {
            Ok(val) => val,
            Err(e) => {
                print::print(&format!("A file error occured during read: {:?}\n", e));
                return 4;
            }
        };
        if count == 0 {
            break;
        }
        out_file.write(&buffer[0..count]);
    }
    f.close();
    0
}

//work in progress

#[no_mangle]
#[inline(never)]
pub extern "C" fn simple_wc(argc: usize, argv: *const &[u8]) -> u32 {
    use crate::syscall::files::File;
    use crate::syscall::*;
    use alloc::vec::Vec;
    use core::convert::TryInto;
    use core::str::from_utf8;

    if argc != 2 && argc != 3 {
        print::print("Invalid number of arguments\n");
        return 1;
    }

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    let option = match core::str::from_utf8(args[0]) {
        Ok(val) => val,
        Err(_) => {
            print::print("Valid options are: -c \n");
            return 2;
        }
    };

    let in_file = if argc == 0 {
        File::get_stdin()
    } else {
        let bytes: &[u8; 8] = match args[1].try_into() {
            Ok(val) => val,
            Err(_) => {
                print::print("Invalid out type pipe source val");
                return 3;
            }
        };
        let pid = u64::from_le_bytes(*bytes);
        print::print(&format!("PID of the beginning of pipe: {}\n", pid));
        set_pipe_read_on_pid(pid);
        File::get_pipein()
    };

    let out_file = if argc == 0 {
        File::get_stdout()
    } else {
        let bytes: &[u8; 8] = match args[2].try_into() {
            Ok(val) => val,
            Err(_) => {
                print::print("Invalid out type flag value");
                return 4;
            }
        };
        if u64::from_le_bytes(*bytes) > 0 {
            File::get_pipeout()
        } else {
            File::get_stdout()
        }
    };

    let mut buffer = [0u8; 32];
    let mut result = Vec::<u8>::new();
    loop {
        match in_file.read(32, &mut buffer) {
            Ok(res) => {
                if res > 0 {
                    result.extend_from_slice(&buffer);
                } else {
                    yield_cpu();
                }
            }
            Err(_) => break,
        };
    }
    in_file.close();

    let string = from_utf8(&result[..]).unwrap().trim_matches(char::from(0));

    let res = match option {
        "-c" => string.chars().count(),
        "-w" => {
            print::print("not implemented yet");
            return 10;
        }
        &_ => {
            print::print("not implemented yet");
            return 10;
        }
    };

    out_file.write(&format!("{}", res).as_bytes());
    0
}

#[link_section = ".task_local"]
static MY_PID: AtomicU64 = AtomicU64::new(0);
#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task(_argc: usize, _argv: *const &[u8]) -> u32 {
    use crate::syscall::asynchronous::files::AsyncFileDescriptor;
    use crate::syscall::files::File;
    use crate::syscall::*;
    use core::str::from_utf8;

    let filename = "file1";
    let cat_to_pipe = 1usize.to_le_bytes();
    let cat_args = [filename.as_bytes(), (&cat_to_pipe) as &[u8]];

    let cat_pid = create_task(simple_cat, &cat_args);

    for _i in 1..10 {
        yield_cpu();
    }

    let pid = cat_pid.to_le_bytes();
    let to_pipe = 1usize.to_le_bytes();
    let wc_args = ["-c".as_bytes(), (&pid) as &[u8], (&to_pipe) as &[u8]];

    let wc_pid = create_task(simple_wc, &wc_args);

    print::print(&format!(
        "Created hello tasks with PIDs: {}, {}\n",
        cat_pid, wc_pid
    ));
    loop {
        let ret_val = get_child_return_value(wc_pid);
        if let Some(ret) = ret_val {
            print::print(&format!("Returned value from wc: {}\n", ret));
            break;
        }
        yield_cpu();
    }

    set_pipe_read_on_pid(wc_pid);

    let mut buff = [0u8; 32];
    let ret = File::get_pipein().read(32, &mut buff);
    if ret.is_err() {
        print::print(&format!(
            "An error occured during the cat {} | wc -c execution",
            filename
        ));
    };
    let string = from_utf8(&buff[..]).unwrap().trim_matches(char::from(0));
    print::print(&format!(
        "The file {} has {} characters\n",
        filename, string
    ));

    create_task(test_async_files, &[]);

    let mut buff = [0u8; 32];
    let ret = File::get_stdin().read(10, &mut buff);
    let string = from_utf8(&buff[..]).unwrap();
    print::print(&format!("FROM STD IN {} \n", string));
    loop {}

    0
}

pub extern "C" fn test_async_files(_argc: usize, _argv: *const &[u8]) -> u32 {
    use crate::syscall::files::File;
    use crate::syscall::*;
    use crate::utils::ONLY_MSB_OF_USIZE;
    use crate::vfs;
    use core::str::from_utf8;

    let submission_buffer = get_async_submission_buffer();
    let completion_buffer = get_async_completion_buffer();

    let mut str_buffer = [0u8; 20];
    let mut str_buffer1 = [0u8; 20];

    File::async_open("file1", true, 1, submission_buffer)
        .then_read(
            20,
            &mut str_buffer as *mut [u8] as *mut u8,
            2,
            submission_buffer,
        )
        .then_seek(-15, vfs::SeekType::FromCurrent, 3, submission_buffer)
        .then_write(b"<Added>", 4, submission_buffer)
        .then_seek(2, vfs::SeekType::FromBeginning, 5, submission_buffer)
        .then_read(
            20,
            &mut str_buffer1 as *mut [u8] as *mut u8,
            6,
            submission_buffer,
        )
        .then_close(7, submission_buffer);

    asynchronous::async_print::async_print("Hello world!\n", 69, submission_buffer);

    loop {
        match asynchronous::async_syscall::get_syscall_returned_value(completion_buffer) {
            Some(val) => {
                print::print(&format!(
                    "Received response for id: {} - {} : {}\n",
                    val.id,
                    val.value,
                    val.value & !ONLY_MSB_OF_USIZE
                ));
                if val.id == 7 {
                    let string = from_utf8(&str_buffer).unwrap();
                    print::print(&format!("1st Read_value: {}\n", string));
                    let string = from_utf8(&str_buffer1).unwrap();
                    print::print(&format!("2nd Read_value: {}\n", string));
                    loop {}
                }
            }
            None => (),
        };
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _loop(_: usize, _: *const &[u8]) -> u32 {
    loop {}
}

pub extern "C" fn pwd(_: usize, _: *const &[u8]) -> u32 {
    print("/\n");
    0
}
pub extern "C" fn clear(_: usize, _: *const &[u8]) -> u32 {
    print("\x1B[2J\x1B[2;1H\x1B[2J\x1B[2;1H");
    0
}

type Program = (&'static str, extern "C" fn(usize, *const &[u8]) -> u32);

const PROGRAMS: [Program; 10] = [
    ("shell", shell),
    ("loop", _loop),
    ("first_task", first_task),
    ("test_async_files", test_async_files),
    ("simple_wc", simple_wc),
    ("simple_cat", simple_cat),
    ("true", _true),
    ("false", _false),
    ("pwd", pwd),
    ("clear", clear),
];
#[derive(Debug)]
enum ParseError {
    UnknownProgram(String),
    QuoteUnclosed,
}

pub extern "C" fn shell(argc: usize, argv: *const &[u8]) -> u32 {
    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    match shell_impl(args) {
        Ok(_) => 0,
        Err(error_code) => error_code,
    }
}

use crate::syscall::files::File;
use crate::syscall::print::print;
use crate::syscall::{create_task, get_child_return_value, yield_cpu};
use alloc::vec::Vec;

type ErrorCode = u32;

const UTF8_ERROR: u32 = 20;

const READ_ERROR: u32 = 30;

fn shell_impl(args: &[&[u8]]) -> Result<(), ErrorCode> {
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

fn add_new_input(command_line : &mut String, new_input : &str) {
    let mut new_input : String = new_input.into();

    let (finished, deletions ) = process_input(&mut new_input);

    *command_line += &new_input;
    if !finished {
        let (finished, new_deletions ) = process_input(command_line);
        if !finished {
            if let Some(index) = command_line.find(|c| c != '\u{7f}') {
                let (_, tail) = command_line.split_at(index);
                *command_line = tail.into();
            }
            else {
                *command_line = String::from("");
            }
        }
        if new_deletions > 0 {
            print(&format!("\x1B[{}D\x1B[0K", new_deletions));
        }
        print(&new_input);
    }else {
        print(&new_input);
    }
}


/// returns pair of flag that is true if string was processed completly and count of removed characters
fn process_input(string : &mut String) -> (bool,usize) {
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
            },
            ' '..='~' | '\n' => cleared.push(c),
            '\u{1b}' => {
                skip = 2;
            }
            _ => {},
        }
    }
    core::mem::swap(string, &mut cleared);
    (finished,deletions)
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
        for command in indivdual_comands {
            match run_command(command) {
                Ok(return_code) => print(&format!("Program exited with code {}\n", return_code)),
                Err(parse_error) => print(&format!("Shell Error: {:?}\n", parse_error)),
            }
        }

        // print(&format!("cmd: {}\n", base_cmd));
    }
    Ok(0)
}
fn run_command(command: &str) -> Result<ErrorCode, ParseError> {
    let words = command.shell_split("\'\"".chars(), " ".chars())?;
    let (head, tail) = words.split_at(1);
    let command_name = head[0];

    let bytes: Vec<&[u8]> = tail.iter().map(|string| string.as_bytes()).collect();

    for &(name, function) in PROGRAMS.iter() {
        if name == command_name {
            let child_pid = create_task(function, &bytes[..]);
            let return_val = await_child(child_pid);
            return Ok(return_val);
        }
    }
    Err(ParseError::UnknownProgram(command.into()))
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
    print("\u{1FA90} default@uranos | \u{1F5C1}  / > ");
}
