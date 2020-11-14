use crate::aarch64::asm;
use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, Ordering};

/// Spinlock based userspace Mutex type for allowing concurent access to protected data
pub struct Yutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Sync> Sync for Yutex<T> {}
unsafe impl<T: Send> Send for Yutex<T> {}

/// RAII Lock guard for Yutex type
pub struct YutexGuard<'a, T> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}
impl<'a, T> Deref for YutexGuard<'a, T> {
    type Target = T;
    fn deref(&'_ self) -> &'_ T {
        &*self.data
    }
}
impl<'a, T> DerefMut for YutexGuard<'a, T> {
    fn deref_mut(&'_ mut self) -> &'_ mut T {
        &mut *self.data
    }
}
impl<'a, T> Drop for YutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
impl<T> Yutex<T> {
    ///Crates new Yutex around provided data
    pub const fn new(data: T) -> Self {
        Yutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    /// Locks the Yutex and returns dereferncable lock guard
    /// For most cases prefer using sync method
    ///
    pub fn lock(&self) -> YutexGuard<T> {
        self.take_lock();
        YutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
    /// Locks Yutex and passes prtoected value to provided function F
    /// Releases Yutex afterwards, ad returns rusult of function F
    /// Prefered way of accesing data under Yutex
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
                crate::syscall::yield_cpu();
            }
        }
    }
}
