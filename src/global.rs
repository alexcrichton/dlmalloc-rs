use core::alloc::{GlobalAlloc, Layout};
use core::ops::{Deref, DerefMut};

use Dlmalloc;

/// An instance of a "global allocator" backed by `Dlmalloc`
///
/// This API requires the `global` feature is activated, and this type
/// implements the `GlobalAlloc` trait in the standard library.
pub struct GlobalDlmalloc;

impl GlobalDlmalloc {
    /// Lock the global allocator.
    ///
    /// Any attempt to allocate using this allocator will block the calling thread,
    /// until the allocator is unlocked.
    #[inline]
    pub fn lock(&self) {
        ::sys::acquire_global_lock()
    }

    ///  Unlock the global allocator.
    ///
    /// # Safety
    ///
    /// This method may only be called if the allocator is currently locked.
    #[inline]
    pub unsafe fn unlock(&self) {
        ::sys::release_global_lock()
    }
}

unsafe impl GlobalAlloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc>::malloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        <Dlmalloc>::free(&mut get(), ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc>::calloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        <Dlmalloc>::realloc(&mut get(), ptr, layout.size(), layout.align(), new_size)
    }
}

static mut DLMALLOC: Dlmalloc = Dlmalloc::new();

struct Instance;

unsafe fn get() -> Instance {
    ::sys::acquire_global_lock();
    Instance
}

impl Deref for Instance {
    type Target = Dlmalloc;
    fn deref(&self) -> &Dlmalloc {
        unsafe { &DLMALLOC }
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Dlmalloc {
        unsafe { &mut DLMALLOC }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        ::sys::release_global_lock()
    }
}
