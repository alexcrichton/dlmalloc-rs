use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use System;

// Do not remove inline: will result in relocation failure
#[inline(always)]
unsafe fn rel_ptr_mut<T>(offset: u64) -> *mut T {
    (image_base() + offset) as *mut T
}

// Do not remove inline: will result in relocation failure
// For the same reason we use inline ASM here instead of an extern static to
// locate the base
#[inline(always)]
fn image_base() -> u64 {
    let base;
    unsafe { llvm_asm!("lea IMAGE_BASE(%rip),$0":"=r"(base)) };
    base
}

/// System interface implementation for SGX platform
pub struct Platform;

impl System for Platform {
    /// Allocs system resources
    unsafe fn alloc(_size: usize) -> (*mut u8, usize, u32) {
        extern "C" {
            static HEAP_BASE: u64;
            static HEAP_SIZE: usize;
        }
        static INIT: AtomicBool = AtomicBool::new(false);
        // No ordering requirement since this function is protected by the global lock.
        if !INIT.swap(true, Ordering::Relaxed) {
            (rel_ptr_mut(HEAP_BASE), HEAP_SIZE, 0)
        } else {
            (ptr::null_mut(), 0, 0)
        }
    }

    unsafe fn remap(_ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
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
        false
    }

    fn page_size() -> usize {
        0x1000
    }
}
