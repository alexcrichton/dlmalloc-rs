#![feature(allocator_api, alloc)]
#![no_std]

extern crate alloc;

use alloc::heap::{Alloc, Layout, AllocErr};

pub use self::global::GlobalDlmalloc;

mod dlmalloc;

pub struct Dlmalloc(dlmalloc::Dlmalloc);

#[cfg(unix)]
#[path = "unix.rs"]
mod sys;
mod global;

#[repr(C)]
struct Header(*mut u8);

unsafe fn get_header<'a>(ptr: *mut u8) -> &'a mut Header {
    &mut *(ptr as *mut Header).offset(-1)
}

unsafe fn align_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
    let aligned = ptr.offset((align - (ptr as usize & (align - 1))) as isize);
    *get_header(aligned) = Header(ptr);
    aligned
}

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
            let size = layout.size() + layout.align();
            let ptr = self.0.malloc(size);
            if ptr.is_null() {
                ptr
            } else {
                align_ptr(ptr, layout.align())
            }
        };
        if ptr.is_null() {
            Err(AllocErr::Exhausted { request: layout })
        } else {
            Ok(ptr)
        }
    }

    // #[inline]
    // unsafe fn alloc_zeroed(&mut self, layout: Layout)
    //     -> Result<*mut u8, AllocErr>
    // {
    //     (&*self).alloc_zeroed(layout)
    // }
    //
    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= self.0.malloc_alignment() {
            self.0.free(ptr)
        } else {
            let header = get_header(ptr);
            self.0.free(header.0)
        }
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
