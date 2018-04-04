#![feature(allocator_api, nonnull_cast)]
#![cfg_attr(target_arch = "wasm32", feature(link_llvm_intrinsics))]
#![no_std]

use core::alloc::{Alloc, Layout, AllocErr, Void};
use core::cmp;
use core::ptr::{self, NonNull};

pub use self::global::GlobalDlmalloc;

mod global;
mod dlmalloc;

pub struct Dlmalloc(dlmalloc::Dlmalloc);

#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
mod sys;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod sys;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod sys;

impl Dlmalloc {
    pub fn new() -> Dlmalloc {
        Dlmalloc(dlmalloc::Dlmalloc::new())
    }
}

impl Dlmalloc {
    #[inline]
    unsafe fn ptr_alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.align() <= self.0.malloc_alignment() {
            self.0.malloc(layout.size())
        } else {
            self.0.memalign(layout.align(), layout.size())
        }
    }

    #[inline]
    unsafe fn ptr_alloc_zeroed(&mut self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let ptr = self.ptr_alloc(layout);
        if !ptr.is_null() {
            if self.0.calloc_must_clear(ptr) {
                ptr::write_bytes(ptr, 0, size);
            }
        }
        ptr
    }

    #[inline]
    unsafe fn ptr_dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        drop(layout);
        self.0.free(ptr)
    }

    #[inline]
    unsafe fn ptr_realloc(&mut self,
                          ptr: *mut u8,
                          layout: Layout,
                          new_size: usize) -> *mut u8 {
        if layout.align() <= self.0.malloc_alignment() {
            self.0.realloc(ptr, new_size)
        } else {
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_ptr = self.ptr_alloc(new_layout);
            if !new_ptr.is_null() {
                let size = cmp::min(layout.size(), new_size);
                ptr::copy_nonoverlapping(ptr, new_ptr, size);
                self.ptr_dealloc(ptr, layout);
            }
            new_ptr
        }
    }
}

fn to_result(ptr: *mut u8) -> Result<NonNull<Void>, AllocErr> {
    NonNull::new(ptr as *mut Void).ok_or(AllocErr)
}

unsafe impl Alloc for Dlmalloc {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<Void>, AllocErr> {
        to_result(self.ptr_alloc(layout))
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<Void>, AllocErr>
    {
        to_result(self.ptr_alloc_zeroed(layout))
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<Void>, layout: Layout) {
        self.ptr_dealloc(ptr.cast().as_ptr(), layout)
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: NonNull<Void>,
                      layout: Layout,
                      new_size: usize) -> Result<NonNull<Void>, AllocErr> {
        to_result(self.ptr_realloc(ptr.cast().as_ptr(), layout, new_size))
    }
}
