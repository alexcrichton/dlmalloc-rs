use core::arch::wasm32;
use core::ptr;

pub unsafe fn alloc(size: usize) -> (*mut u8, usize, u32) {
    let pages = size / page_size();
    let prev = wasm32::memory_grow(0, pages);
    if prev == usize::max_value() {
        return (ptr::null_mut(), 0, 0);
    }
    ((prev * page_size()) as *mut u8, pages * page_size(), 0)
}

pub unsafe fn remap(_ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
    // TODO: I think this can be implemented near the end?
    ptr::null_mut()
}

pub unsafe fn free_part(_ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
    false
}

pub unsafe fn free(_ptr: *mut u8, _size: usize) -> bool {
    false
}

pub fn can_release_part(_flags: u32) -> bool {
    false
}

#[cfg(feature = "global")]
pub fn acquire_global_lock() {
    // single threaded, no need!
}

#[cfg(feature = "global")]
pub fn release_global_lock() {
    // single threaded, no need!
}

pub fn allocates_zeros() -> bool {
    true
}

pub fn page_size() -> usize {
    64 * 1024
}
