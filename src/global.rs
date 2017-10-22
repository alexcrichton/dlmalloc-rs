use alloc::heap::{Alloc, Layout, Excess, CannotReallocInPlace, AllocErr};
use core::ops::{Deref, DerefMut};

use Dlmalloc;

pub struct GlobalDlmalloc;

unsafe impl Alloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        (&*self).alloc(layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout)
        -> Result<*mut u8, AllocErr>
    {
        (&*self).alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        (&*self).dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      old_layout: Layout,
                      new_layout: Layout) -> Result<*mut u8, AllocErr> {
        (&*self).realloc(ptr, old_layout, new_layout)
    }

    // fn oom(&mut self, err: AllocErr) -> ! {
    //     (&*self).oom(err)
    // }

    #[inline]
    fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        (&self).usable_size(layout)
    }

    #[inline]
    unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
        (&*self).alloc_excess(layout)
    }

    #[inline]
    unsafe fn realloc_excess(&mut self,
                             ptr: *mut u8,
                             layout: Layout,
                             new_layout: Layout) -> Result<Excess, AllocErr> {
        (&*self).realloc_excess(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn grow_in_place(&mut self,
                            ptr: *mut u8,
                            layout: Layout,
                            new_layout: Layout) -> Result<(), CannotReallocInPlace> {
        (&*self).grow_in_place(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn shrink_in_place(&mut self,
                              ptr: *mut u8,
                              layout: Layout,
                              new_layout: Layout) -> Result<(), CannotReallocInPlace> {
        (&*self).shrink_in_place(ptr, layout, new_layout)
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

unsafe impl<'a> Alloc for &'a GlobalDlmalloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        get().alloc(layout)
    }

    unsafe fn alloc_zeroed(&mut self, layout: Layout)
        -> Result<*mut u8, AllocErr>
    {
        get().alloc_zeroed(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        get().dealloc(ptr, layout)
    }

    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      old_layout: Layout,
                      new_layout: Layout) -> Result<*mut u8, AllocErr> {
        get().realloc(ptr, old_layout, new_layout)
    }

    // fn oom(&mut self, err: AllocErr) -> ! {
    //     unsafe { get().oom(err) }
    // }

    fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        unsafe { get().usable_size(layout) }
    }

    #[inline]
    unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
        get().alloc_excess(layout)
    }

    #[inline]
    unsafe fn realloc_excess(&mut self,
                             ptr: *mut u8,
                             layout: Layout,
                             new_layout: Layout) -> Result<Excess, AllocErr> {
        get().realloc_excess(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn grow_in_place(&mut self,
                            ptr: *mut u8,
                            layout: Layout,
                            new_layout: Layout) -> Result<(), CannotReallocInPlace> {
        get().grow_in_place(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn shrink_in_place(&mut self,
                              ptr: *mut u8,
                              layout: Layout,
                              new_layout: Layout) -> Result<(), CannotReallocInPlace> {
        get().shrink_in_place(ptr, layout, new_layout)
    }
}
