use aarch64::asm;
use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, Ordering};

/// Spinlock based Mutex type for allowing concurent access to protected data
pub struct Mutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}
/// RAII Lock guard for mutex type
pub struct MutexGuard<'a, T> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&'_ self) -> &'_ T {
        &*self.data
    }
}
impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&'_ mut self) -> &'_ mut T {
        &mut *self.data
    }
}
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
impl<T> Mutex<T> {
    ///Crates new mutex around provided data
    pub fn new(data: T) -> Self {
        Mutex {
            lock: AtomicBool::from(false),
            data: UnsafeCell::new(data),
        }
    }
    /// Locks the mutex and returns dereferncable lock guard
    /// For most cases prefer using sync method
    ///
    pub unsafe fn lock(&self) -> MutexGuard<T> {
        self.take_lock();
        MutexGuard {
            lock: &self.lock,
            data: &mut *self.data.get(),
        }
    }
    /// Locks mutex and passes prtoected value to provided function F
    /// Releases Mutex afterwards, ad returns rusult of function F
    /// Prefered way of accesing data under mutex
    pub fn sync<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.take_lock();

        let result = f(unsafe { &mut *self.data.get() });

        self.lock.store(false, Ordering::Release);
        result
    }

    fn take_lock(&self) {
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) {
            // Wait until the lock seems unlocked
            while self.lock.load(Ordering::Relaxed) {
                asm::nop();
            }
        }
    }
}
