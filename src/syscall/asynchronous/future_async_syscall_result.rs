#[derive(Debug)]
pub struct FutureAsyncSyscallResult<T> {
    pub response: Option<T>,
    pub is_finished: bool,
}

impl<T> FutureAsyncSyscallResult<T> {
    pub fn new() -> Self {
        FutureAsyncSyscallResult::<T> {
            response: None,
            is_finished: false,
        }
    }
    pub fn is_done(&self) -> bool {
        self.is_finished
    }
    pub fn set_done(&mut self) {
        crate::println!("NO I GITT");
        self.is_finished = true;
    }
    pub fn poll(&self) -> Option<&T> {
        if self.is_finished {
            self.response.as_ref()
        } else {
            None
        }
    }
    pub fn r#await(&self) -> &T {
        loop {
            if self.is_finished {
                return self.response.as_ref().unwrap();
            }
            crate::syscall::yield_cpu();
        }
    }
}
