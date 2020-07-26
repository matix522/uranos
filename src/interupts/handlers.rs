use crate::interupts::ExceptionContext;
use crate::print;
use crate::println;

fn default_exception_handler(_e: &mut ExceptionContext, source : &str) {
    panic!("Unknown {} Exception type recived.", source);
}

//------------------------------------------------------------------------------
// Current, EL0
//------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_el0_synchronous");
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_el0_irq");
}

#[no_mangle]
unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_el0_serror");
}

//------------------------------------------------------------------------------
// Current, ELx
//------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    e.elr_el1 |= crate::KERNEL_OFFSET as u64;
    // crate::println!("LR:  {:x}", e.elr_el1);
    // crate::println!("FAR: {:x}", e.far_el1);
    // crate::println!("x0:  {:x}", e.gpr[0]);
    e.elr_el1 = e.gpr[0] |  crate::KERNEL_OFFSET as u64;
    // default_exception_handler(e, "current_elx_synchronous");
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    println!("Received current_elx_irq interrupt");
    default_exception_handler(e, "current_elx_irq");
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_elx_serror");
}

//------------------------------------------------------------------------------
// Lower, AArch64
//------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    e.elr_el1 |= crate::KERNEL_OFFSET as u64;
    crate::println!("LR: {:x}", e.elr_el1);
    crate::println!("x0: {:x}", e.gpr[0]);
    default_exception_handler(e, "lower_aarch64_synchronous")

}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch64_irq");
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch64_serror");
}

//------------------------------------------------------------------------------
// Lower, AArch32
//------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_synchronous");
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_irq");
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_serror");
}
