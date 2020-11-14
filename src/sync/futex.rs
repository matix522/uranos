use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, Ordering};

/// Spinlock based userspace Mutex type for allowing concurent access to protected data
pub struct Futex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Sync> Sync for Futex<T> {}
unsafe impl<T: Send> Send for Futex<T> {}

/// RAII Lock guard for Futex type
pub struct FutexGuard<'a, T> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}
impl<'a, T> Deref for FutexGuard<'a, T> {
    type Target = T;
    fn deref(&'_ self) -> &'_ T {
        &*self.data
    }
}
impl<'a, T> DerefMut for FutexGuard<'a, T> {
    fn deref_mut(&'_ mut self) -> &'_ mut T {
        &mut *self.data
    }
}
impl<'a, T> Drop for FutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
impl<T> Futex<T> {
    ///Crates new Futex around provided data
    pub const fn new(data: T) -> Self {
        Futex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    /// Locks the Futex and returns dereferncable lock guard
    /// For most cases prefer using sync method
    ///
    pub fn lock(&self) -> FutexGuard<T> {
        self.take_lock();
        FutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
    /// Locks Futex and passes prtoected value to provided function F
    /// Releases Futex afterwards, ad returns rusult of function F
    /// Prefered way of accesing data under Futex
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
                crate::scheduler::switch_task();
            }
        }
    }
}
