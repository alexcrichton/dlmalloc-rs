use core::alloc::{Alloc, Layout, GlobalAlloc, AllocErr, Opaque};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use Dlmalloc;

pub struct GlobalDlmalloc;

unsafe impl GlobalAlloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        <Dlmalloc>::malloc(&mut get(), layout.size(), layout.align()) as *mut Opaque
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        <Dlmalloc>::free(&mut get(), ptr as *mut u8, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut Opaque {
        <Dlmalloc>::calloc(&mut get(), layout.size(), layout.align()) as *mut Opaque
    }

    #[inline]
    unsafe fn realloc(
        &self,
        ptr: *mut Opaque,
        layout: Layout,
        new_size: usize
    ) -> *mut Opaque {
        <Dlmalloc>::realloc(
            &mut get(),
            ptr as *mut u8,
            layout.size(),
            layout.align(),
            new_size,
        ) as *mut Opaque
    }
}

unsafe impl Alloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(
        &mut self,
        layout: Layout
    ) -> Result<NonNull<Opaque>, AllocErr> {
        get().alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout) {
        get().dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: NonNull<Opaque>,
        layout: Layout,
        new_size: usize
    ) -> Result<NonNull<Opaque>, AllocErr> {
        Alloc::realloc(&mut *get(), ptr, layout, new_size)
    }

    #[inline]
    unsafe fn alloc_zeroed(
        &mut self,
        layout: Layout
    ) -> Result<NonNull<Opaque>, AllocErr> {
        get().alloc_zeroed(layout)
    }
}

static mut DLMALLOC: Dlmalloc = Dlmalloc(::dlmalloc::DLMALLOC_INIT);

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
