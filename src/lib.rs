#![feature(allocator_api, alloc)]
#![cfg_attr(target_arch = "wasm32", feature(link_llvm_intrinsics))]
#![no_std]

extern crate alloc;

use alloc::heap::{Alloc, Layout, AllocErr};
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

unsafe impl Alloc for Dlmalloc {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let ptr = if layout.align() <= self.0.malloc_alignment() {
            self.0.malloc(layout.size())
        } else {
            self.0.memalign(layout.align(), layout.size())
        };
        NonNull::new(ptr).ok_or_else(|| AllocErr::Exhausted { request: layout })
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout)
        -> Result<NonNull<u8>, AllocErr>
    {
        let size = layout.size();
        let ptr = self.alloc(layout)?;
        if self.0.calloc_must_clear(ptr.as_ptr()) {
            ptr::write_bytes(ptr.as_ptr(), 0, size);
        }
        Ok(ptr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        drop(layout);
        self.0.free(ptr.as_ptr())
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: NonNull<u8>,
                      old_layout: Layout,
                      new_layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        if old_layout.align() != new_layout.align() {
            return Err(AllocErr::Unsupported {
                details: "cannot change alignment on `realloc`",
            })
        }

        if new_layout.align() <= self.0.malloc_alignment() {
            let ptr = self.0.realloc(ptr.as_ptr(), new_layout.size());
            NonNull::new(ptr).ok_or_else(|| AllocErr::Exhausted { request: new_layout })
        } else {
            let res = self.alloc(new_layout.clone());
            if let Ok(new_ptr) = res {
                let size = cmp::min(old_layout.size(), new_layout.size());
                ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), size);
                self.dealloc(ptr, old_layout);
            }
            res
        }
    }

    // fn oom(&mut self, err: AllocErr) -> ! {
    //     System.oom(err)
    // }

    // #[inline]
    // fn usable_size(&self, layout: &Layout) -> (usize, usize) {
    //     (&self).usable_size(layout)
    // }
    //
    // #[inline]
    // unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
    //     (&*self).alloc_excess(layout)
    // }
    //
    // #[inline]
    // unsafe fn realloc_excess(&mut self,
    //                          ptr: NonNull<u8>,
    //                          layout: Layout,
    //                          new_layout: Layout) -> Result<Excess, AllocErr> {
    //     (&*self).realloc_excess(ptr, layout, new_layout)
    // }
    //
    // #[inline]
    // unsafe fn grow_in_place(&mut self,
    //                         ptr: NonNull<u8>,
    //                         layout: Layout,
    //                         new_layout: Layout) -> Result<(), CannotReallocInPlace> {
    //     (&*self).grow_in_place(ptr, layout, new_layout)
    // }
    //
    // #[inline]
    // unsafe fn shrink_in_place(&mut self,
    //                           ptr: NonNull<u8>,
    //                           layout: Layout,
    //                           new_layout: Layout) -> Result<(), CannotReallocInPlace> {
    //     (&*self).shrink_in_place(ptr, layout, new_layout)
    // }
}

// unsafe impl<'a> Alloc for &'a Dlmalloc {
//     #[inline]
//     unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
//         panic!()
//     }
//
//     // #[inline]
//     // unsafe fn alloc_zeroed(&mut self, layout: Layout)
//     //     -> Result<NonNull<u8>, AllocErr>
//     // {
//     //     panic!()
//     // }
//
//     #[inline]
//     unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
//         panic!()
//     }
//
//     // #[inline]
//     // unsafe fn realloc(&mut self,
//     //                   ptr: NonNull<u8>,
//     //                   old_layout: Layout,
//     //                   new_layout: Layout) -> Result<NonNull<u8>, AllocErr> {
//     //     panic!()
//     // }
//
//     fn oom(&mut self, err: AllocErr) -> ! {
//         System.oom(err)
//     }
// }
