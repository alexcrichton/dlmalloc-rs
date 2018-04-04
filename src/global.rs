use core::alloc::{GlobalAlloc, Layout, Void};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use Dlmalloc;

pub struct GlobalDlmalloc;

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

unsafe impl GlobalAlloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut Void {
        get().ptr_alloc(layout) as *mut Void
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut Void {
        get().ptr_alloc_zeroed(layout) as *mut Void
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut Void, layout: Layout) {
        get().ptr_dealloc(ptr as *mut u8, layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut Void, layout: Layout, new_size: usize) -> *mut Void {
        get().ptr_realloc(ptr as *mut u8, layout, new_size) as *mut Void
    }
}
