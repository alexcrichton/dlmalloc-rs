use core::ptr;
use core::ffi::{c_void};
use once_cell::sync::Lazy;
use Allocator;

pub struct System {
    _priv: (),
}


impl System {
    pub const fn new() -> System { System { _priv: () } }
}

unsafe impl Allocator for System {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
            let addr = unsafe {
                windows::Win32::System::Memory::VirtualAlloc(
                    ptr::null_mut(),
                    size,
                    windows::Win32::System::Memory::MEM_RESERVE | windows::Win32::System::Memory::MEM_COMMIT,
                    windows::Win32::System::Memory::PAGE_READWRITE,
                )
            };

            if addr == ptr::null_mut() {
                (ptr::null_mut(), 0, 0)
            } else {
                (addr as *mut u8, size, 0)
            }
    }

    fn remap(&self, ptr: *mut u8, oldsize: usize, newsize: usize, can_move: bool) -> *mut u8 {
        ptr::null_mut()
    }

    fn free_part(&self, ptr: *mut u8, oldsize: usize, newsize: usize) -> bool { false }

    fn free(&self, ptr: *mut u8, size: usize) -> bool {
        unsafe {
            windows::Win32::System::Memory::VirtualFree(
                ptr as *mut c_void,
                0,
                windows::Win32::System::Memory::MEM_RELEASE).0 != 0
        }
    }

    fn can_release_part(&self, _flags: u32) -> bool { true }

    fn allocates_zeros(&self) -> bool { true }

    fn page_size(&self) -> usize { 4096 }
}

#[cfg(feature = "global")]
static LOCK: Lazy<windows::Win32::Foundation::HANDLE> = unsafe {
    Lazy::new(|| {
        windows::Win32::System::Threading::CreateMutexA(
            ptr::null_mut(),
            false,
            windows::core::PCSTR::default())
    })
};

#[cfg(feature = "global")]
pub fn acquire_global_lock() {
    unsafe { assert_ne!(windows::Win32::System::Threading::WaitForSingleObject(*LOCK, u32::MAX), u32::MAX) };
}

#[cfg(feature = "global")]
pub fn release_global_lock() {
    unsafe { assert_ne!(windows::Win32::System::Threading::ReleaseMutex(*LOCK).0, 0) };
}

/// Under consideration
#[cfg(feature = "global")]
pub unsafe fn enable_alloc_after_fork() {
    // TODO(threadedstream): do I need to implement it?
}
