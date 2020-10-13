use crate::{GlobalSystem, Platform};
#[cfg(feature = "allocator-api")]
use core::alloc::{AllocErr, AllocRef};
use core::alloc::{GlobalAlloc, Layout};
use core::ops::{Deref, DerefMut};
#[cfg(feature = "allocator-api")]
use core::ptr::NonNull;

use Dlmalloc;

/// An instance of a "global allocator" backed by `Dlmalloc`
///
/// This API requires the `global` feature is activated, and this type
/// implements the `GlobalAlloc` trait in the standard library.
pub struct GlobalDlmalloc;

unsafe impl GlobalAlloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc<Platform>>::malloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        <Dlmalloc<Platform>>::free(&mut get(), ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc<Platform>>::calloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        <Dlmalloc<Platform>>::realloc(&mut get(), ptr, layout.size(), layout.align(), new_size)
    }
}

#[cfg(feature = "allocator-api")]
unsafe impl AllocRef for GlobalDlmalloc {
    #[inline]
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        unsafe { get().alloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        get().dealloc(ptr, layout)
    }

    #[inline]
    fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        unsafe { get().alloc_zeroed(layout) }
    }
}

static mut DLMALLOC: Dlmalloc<Platform> = Dlmalloc::new();

struct Instance;

unsafe fn get() -> Instance {
    Platform::acquire_global_lock();
    Instance
}

impl Deref for Instance {
    type Target = Dlmalloc<Platform>;
    fn deref(&self) -> &Dlmalloc<Platform> {
        unsafe { &DLMALLOC }
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Dlmalloc<Platform> {
        unsafe { &mut DLMALLOC }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        Platform::release_global_lock()
    }
}
