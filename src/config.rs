use core::sync::atomic::{AtomicBool, Ordering};

static USE_USER_SPACE: AtomicBool = AtomicBool::new(false);
pub fn use_user_space() -> bool {
    USE_USER_SPACE.load(Ordering::SeqCst)
}
pub fn set_use_user_space(value: bool) {
    USE_USER_SPACE.store(value, Ordering::SeqCst);
}

pub fn page_size() -> usize {
    4096 // 4 KiB
}
static DEBUG_ALLOC: AtomicBool = AtomicBool::new(false);

pub fn debug_alloc() -> bool {
    DEBUG_ALLOC.load(Ordering::SeqCst)
}

pub fn set_debug_alloc(value: bool) {
    DEBUG_ALLOC.store(value, Ordering::SeqCst);
}

static DEBUG_MMU: AtomicBool = AtomicBool::new(false);

pub fn debug_mmu() -> bool {
    DEBUG_MMU.load(Ordering::SeqCst)
}

pub fn set_debug_mmu(value: bool) {
    DEBUG_MMU.store(value, Ordering::SeqCst);
}

static MUTEX_HACK: AtomicBool = AtomicBool::new(false);
pub fn debug_mutex() -> bool {
    MUTEX_HACK.load(Ordering::Relaxed)
}

pub fn set_debug_mutex(value: bool) {
    MUTEX_HACK.store(value, Ordering::Relaxed);
}
