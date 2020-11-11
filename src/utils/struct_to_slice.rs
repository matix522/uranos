pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of::<T>())
}

pub unsafe fn u8_slice_to_any<T: Sized>(slice: &[u8]) -> &T {
    let (head, body, _tail) = slice.align_to::<T>();
    assert!(head.is_empty(), "Data was not aligned");
    &body[0]
}

pub unsafe fn u8_slice_to_any_mut<T: Sized>(slice: &mut [u8]) -> &mut T {
    let (head, body, _tail) = slice.align_to_mut::<T>();
    assert!(head.is_empty(), "Data was not aligned");
    &mut body[0]
}
