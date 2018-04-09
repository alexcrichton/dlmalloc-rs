#![cfg_attr(feature = "allocator-api", feature(allocator_api, alloc))]
#![cfg_attr(target_arch = "wasm32", feature(link_llvm_intrinsics))]
#![cfg_attr(not(feature = "allocator-api"), allow(dead_code))]
#![no_std]

#[cfg(feature = "allocator-api")]
extern crate alloc;

#[cfg(feature = "allocator-api")]
use alloc::heap::{Alloc, Layout, AllocErr};
use core::cmp;
use core::ptr;

#[cfg(feature = "allocator-api")]
pub use self::global::GlobalDlmalloc;

#[cfg(feature = "allocator-api")]
mod global;
mod dlmalloc;

pub struct Dlmalloc(dlmalloc::Dlmalloc);

pub const DLMALLOC_INIT: Dlmalloc = Dlmalloc(dlmalloc::DLMALLOC_INIT);

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

    #[inline]
    pub unsafe fn malloc(&mut self, size: usize, align: usize) -> *mut u8 {
        if align <= self.0.malloc_alignment() {
            self.0.malloc(size)
        } else {
            self.0.memalign(align, size)
        }
    }

    #[inline]
    pub unsafe fn calloc(&mut self, size: usize, align: usize) -> *mut u8 {
        let ptr = self.malloc(size, align);
        if !ptr.is_null() && self.0.calloc_must_clear(ptr) {
            ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }

    #[inline]
    pub unsafe fn free(&mut self, ptr: *mut u8, size: usize, align: usize) {
        drop((size, align));
        self.0.free(ptr)
    }

    #[inline]
    pub unsafe fn realloc(&mut self,
                          ptr: *mut u8,
                          old_size: usize,
                          old_align: usize,
                          new_size: usize) -> *mut u8 {
        if old_align <= self.0.malloc_alignment() {
            self.0.realloc(ptr, new_size)
        } else {
            let res = self.malloc(new_size, old_align);
            if !res.is_null() {
                let size = cmp::min(old_size, new_size);
                ptr::copy_nonoverlapping(ptr, res, size);
                self.free(ptr, old_size, old_align);
            }
            res
        }
    }
}

#[cfg(feature = "allocator-api")]
unsafe impl Alloc for Dlmalloc {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let ptr = <Dlmalloc>::malloc(self, layout.size(), layout.align());
        if ptr.is_null() {
            Err(AllocErr::Exhausted { request: layout })
        } else {
            Ok(ptr)
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout)
        -> Result<*mut u8, AllocErr>
    {
        let ptr = <Dlmalloc>::calloc(self, layout.size(), layout.align());
        if ptr.is_null() {
            Err(AllocErr::Exhausted { request: layout })
        } else {
            Ok(ptr)
        }
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        <Dlmalloc>::free(self, ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      old_layout: Layout,
                      new_layout: Layout) -> Result<*mut u8, AllocErr> {
        if old_layout.align() != new_layout.align() {
            return Err(AllocErr::Unsupported {
                details: "cannot change alignment on `realloc`",
            })
        }
        let ptr = <Dlmalloc>::realloc(
            self,
            ptr,
            old_layout.size(),
            old_layout.align(),
            new_layout.size(),
        );


        if ptr.is_null() {
            Err(AllocErr::Exhausted { request: new_layout })
        } else {
            Ok(ptr)
        }
    }
}
