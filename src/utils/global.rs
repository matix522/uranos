// use core::cell::UnsafeCell;
// use core::hint::unreachable_unchecked;
// use core::ops::Deref;
// use core::ops::DerefMut;

// pub struct Global<T>{ data : UnsafeCell<Option<T>> }

// impl<T> Deref for Global<T>{
//     type Target = Option<&'static T>;
//     fn deref(&self) -> Option<&'static T> {
//         (&*self.ptr()).as_ref()
//     }
// }
// impl<T> DerefMut for Global<T>{
//     //type Target = Option<&'static mut T>;
//     fn deref_mut(&self) -> Option<&'static mut T> {
//         (&*self.ptr()).as_mut()
//     }
// }
// impl<T> Global<T> {
//     pub const fn new() -> Self {
//         Global {data : UnsafeCell::new(None)}
//     }
//     pub const fn from(data : T) -> Self {
//         Global {data : UnsafeCell::new(Some(data))}
//     }
//     fn ptr(&self) -> *mut Option<T> {
//         self.data.get()
//     }
//     pub unsafe fn as_ref_unchecked(&self) -> &'static T {
//        match (&*self.ptr()).as_ref() {
//            Some(ref data) => data,
//            None => unreachable_unchecked(),
//        }
//     }

//     pub unsafe fn as_mut_unchecked(&self) -> &'static mut T {
//         match (&*self.ptr()).as_mut() {
//            Some(data) => data,
//            None => unreachable_unchecked(),
//        }
//     }
// }

// pub static test : Global<usize> = Global::from(42);
