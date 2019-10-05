//pub use spin::*;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool};
use core::ops::DerefMut;
use core::ops::Deref;

struct Mutex<T> {
    lock : AtomicBool,
    data : UnsafeCell<T>,
}
struct MutexGuard<'a, T> {
    lock : &'a AtomicBool,
    data : &'a mut T,
}
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        & *self.data
    } 
}
impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut *self.data
    } 
}
impl<T> Mutex<T> {
    pub fn new(data : T) -> Self {
        Mutex {
            lock : AtomicBool::from(false),
            data : UnsafeCell::new(data)
        }
    }
    pub fn lock(&self) -> MutexGuard<T> {
        let lock = self.take_lock();
        MutexGuard {
            lock : &self.lock,
            data : unsafe { &mut *self.data.get() }
        }
    }
}