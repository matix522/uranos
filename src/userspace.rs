#[no_mangle]
#[inline(never)]
pub extern "C" fn task_one() {
    loop {
        crate::syscall::print::print("Printing Task One\n");
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn task_two() {
    let mut vec = alloc::vec::Vec::new();
    let mut i = 0;
    loop {
        crate::syscall::print::print("Printing Task Two\n");
        vec.push(i);
        i += 1;
        crate::syscall::print::print(&format!("{:?}", vec.last()));
    }
}
