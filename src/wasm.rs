use core::arch::wasm32;
use core::ptr;
#[cfg(feature = "global")]
use GlobalSystem;
use System;

/// System setting for Wasm
pub struct Platform;

impl System for Platform {
    unsafe fn alloc(size: usize) -> (*mut u8, usize, u32) {
        let pages = size / Self::page_size();
        let prev = wasm32::memory_grow(0, pages);
        if prev == usize::max_value() {
            return (ptr::null_mut(), 0, 0);
        }
        (
            (prev * Self::page_size()) as *mut u8,
            pages * Self::page_size(),
            0,
        )
    }

    unsafe fn remap(_ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        // TODO: I think this can be implemented near the end?
        ptr::null_mut()
    }

    unsafe fn free_part(_ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        false
    }

    unsafe fn free(_ptr: *mut u8, _size: usize) -> bool {
        false
    }

    fn can_release_part(_flags: u32) -> bool {
        false
    }

    fn allocates_zeros() -> bool {
        true
    }

    fn page_size() -> usize {
        64 * 1024
    }
}

#[cfg(feature = "global")]
impl GlobalSystem for Platform {
    fn acquire_global_lock() {
        // single threaded, no need!
    }

    fn release_global_lock() {
        // single threaded, no need!
    }
}
