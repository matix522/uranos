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
