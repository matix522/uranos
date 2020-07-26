pub mod task_context;
pub mod task_stack;

use crate::interupts::ExceptionContext;

pub fn yeet() {
    unsafe {
        llvm_asm!("svc 0");
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn foo() {
    crate::println!("BEHOLD! SECOND TASK");
    loop {}
}

pub fn sample_change_task(_e: &mut ExceptionContext) -> &mut ExceptionContext {
    let task = match task_context::TaskContext::new(foo) {
        Ok(t) => t,
        Err(_) => {
            crate::println!(">>>>>> ERROR CREATING TASK CONTEXT");
            loop {}
        }
    };

    let boxed_task = alloc::boxed::Box::new(task);
    let task_ref: &'static task_context::TaskContext = alloc::boxed::Box::leak(boxed_task);
    // # Safety: this line can be reached only if exeption_context is allocated properly and it's memory is leaked, so it has static lifetime.
    unsafe { &mut *task_ref.exception_context }
}
