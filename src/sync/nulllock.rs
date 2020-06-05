use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;

pub struct NullLock<T> {
    data: UnsafeCell<T>,
}
unsafe impl<T: Sync> Sync for NullLock<T> {}
unsafe impl<T: Send> Send for NullLock<T> {}

pub struct NullLockGuard<'a, T> {
    data: &'a mut T,
}
impl<'a, T> Deref for NullLockGuard<'a, T> {
    type Target = T;
    fn deref(&'_ self) -> &'_ T {
        &*self.data
    }
}
impl<'a, T> DerefMut for NullLockGuard<'a, T> {
    fn deref_mut(&'_ mut self) -> &'_ mut T {
        &mut *self.data
    }
}
impl<'a, T> Drop for NullLockGuard<'a, T> {
    fn drop(&mut self) {}
}
impl<T> NullLock<T> {
    ///Crates new mutex around provided data
    pub const fn new(data: T) -> Self {
        NullLock {
            data: UnsafeCell::new(data),
        }
    }
    /// Locks the mutex and returns dereferncable lock guard
    /// For most cases prefer using sync method
    ///
    pub fn lock(&self) -> NullLockGuard<T> {
        NullLockGuard {
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Locks mutex and passes prtoected value to provided function F
    /// Releases Mutex afterwards, ad returns rusult of function F
    /// Prefered way of accesing data under mutex
    pub fn sync<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(unsafe { &mut *self.data.get() })
    }
}
