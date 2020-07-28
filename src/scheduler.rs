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
fn drop_el0() {
    unsafe {
        llvm_asm!("brk 0");
    };
}
#[no_mangle]
#[inline(never)]
pub extern "C" fn foo() {
    crate::println!(
        "BEHOLD! SECOND TASK ON {:?} LEVEL",
        crate::boot::mode::ExceptionLevel::get_current()
    );
    drop_el0();
    loop {}
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn bar() {
    crate::println!("BEHOLD! NEXT TASK ON USER LEVEL");
    loop {}
}
pub fn sample_change_task(_e: &mut ExceptionContext, is_kernel: bool) -> &mut ExceptionContext {
    let task = match task_context::TaskContext::new(if is_kernel { foo } else { bar }, is_kernel) {
        Ok(t) => t,
        Err(err) => {
            crate::println!(">>>>>> ERROR CREATING TASK CONTEXT {:?}", err);
            loop {}
        }
    };

    let boxed_task = alloc::boxed::Box::new(task);
    let task_ref: &'static task_context::TaskContext = alloc::boxed::Box::leak(boxed_task);
    // # Safety: this line can be reached only if exeption_context is allocated properly and it's memory is leaked, so it has static lifetime.
    unsafe { &mut *task_ref.exception_context }
}
