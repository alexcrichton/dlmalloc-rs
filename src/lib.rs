#![feature(allocator_api, alloc)]
#![no_std]

extern crate alloc;

use alloc::heap::{Alloc, Layout, AllocErr};
use core::ptr;

pub use self::global::GlobalDlmalloc;

mod dlmalloc;

pub struct Dlmalloc(dlmalloc::Dlmalloc);

#[cfg(unix)]
#[path = "unix.rs"]
mod sys;
mod global;

impl Dlmalloc {
    pub fn new() -> Dlmalloc {
        Dlmalloc(dlmalloc::Dlmalloc::new())
    }
}

unsafe impl Alloc for Dlmalloc {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let ptr = if layout.align() <= self.0.malloc_alignment() {
            self.0.malloc(layout.size())
        } else {
            self.0.memalign(layout.align(), layout.size())
        };
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
        let size = layout.size();
        let ptr = self.alloc(layout)?;
        if self.0.calloc_must_clear(ptr) {
            ptr::write_bytes(ptr, 0, size);
        }
        Ok(ptr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        drop(layout);
        self.0.free(ptr)
    }

    // #[inline]
    // unsafe fn realloc(&mut self,
    //                   ptr: *mut u8,
    //                   old_layout: Layout,
    //                   new_layout: Layout) -> Result<*mut u8, AllocErr> {
    //     (&*self).realloc(ptr, old_layout, new_layout)
    // }

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
    //                          ptr: *mut u8,
    //                          layout: Layout,
    //                          new_layout: Layout) -> Result<Excess, AllocErr> {
    //     (&*self).realloc_excess(ptr, layout, new_layout)
    // }
    //
    // #[inline]
    // unsafe fn grow_in_place(&mut self,
    //                         ptr: *mut u8,
    //                         layout: Layout,
    //                         new_layout: Layout) -> Result<(), CannotReallocInPlace> {
    //     (&*self).grow_in_place(ptr, layout, new_layout)
    // }
    //
    // #[inline]
    // unsafe fn shrink_in_place(&mut self,
    //                           ptr: *mut u8,
    //                           layout: Layout,
    //                           new_layout: Layout) -> Result<(), CannotReallocInPlace> {
    //     (&*self).shrink_in_place(ptr, layout, new_layout)
    // }
}

// unsafe impl<'a> Alloc for &'a Dlmalloc {
//     #[inline]
//     unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
//         panic!()
//     }
//
//     // #[inline]
//     // unsafe fn alloc_zeroed(&mut self, layout: Layout)
//     //     -> Result<*mut u8, AllocErr>
//     // {
//     //     panic!()
//     // }
//
//     #[inline]
//     unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
//         panic!()
//     }
//
//     // #[inline]
//     // unsafe fn realloc(&mut self,
//     //                   ptr: *mut u8,
//     //                   old_layout: Layout,
//     //                   new_layout: Layout) -> Result<*mut u8, AllocErr> {
//     //     panic!()
//     // }
//
//     fn oom(&mut self, err: AllocErr) -> ! {
//         System.oom(err)
//     }
// }
