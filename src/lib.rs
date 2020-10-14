//! A Rust port of the `dlmalloc` allocator.
//!
//! The `dlmalloc` allocator is described at
//! http://g.oswego.edu/dl/html/malloc.html and this Rust crate is a straight
//! port of the C code for the allocator into Rust. The implementation is
//! wrapped up in a `Dlmalloc` type and has support for Linux, OSX, and Wasm
//! currently.
//!
//! The primary purpose of this crate is that it serves as the default memory
//! allocator for the `wasm32-unknown-unknown` target in the standard library.
//! Support for other platforms is largely untested and unused, but is used when
//! testing this crate.

#![cfg_attr(feature = "allocator-api", feature(allocator_api))]
#![cfg_attr(target_env = "sgx", feature(llvm_asm))]
#![cfg_attr(not(feature = "allocator-api"), allow(dead_code))]
#![no_std]
#![deny(missing_docs)]

#[cfg(feature = "allocator-api")]
use core::alloc::{AllocErr, AllocRef, Layout};
use core::cmp;
use core::ptr;

#[cfg(all(feature = "global", not(test)))]
pub use self::global::GlobalDlmalloc;

mod dlmalloc;
#[cfg(all(feature = "global", not(test)))]
mod global;

/// A platform interface
#[cfg(any(target_arch = "wasm32", target_os = "macos", target_env = "sgx"))]
pub trait System: Send {
    /// Allocates a memory region of `size` bytes
    unsafe fn alloc(size: usize) -> (*mut u8, usize, u32) {
        sys::alloc(size)
    }

    /// Remaps a memory region
    unsafe fn remap(ptr: *mut u8, oldsize: usize, newsize: usize, can_move: bool) -> *mut u8 {
        sys::remap(ptr, oldsize, newsize, can_move)
    }

    /// Frees a part of a memory region
    unsafe fn free_part(ptr: *mut u8, oldsize: usize, newsize: usize) -> bool {
        sys::free_part(ptr, oldsize, newsize)
    }

    /// Frees an entire memory region
    unsafe fn free(ptr: *mut u8, size: usize) -> bool {
        sys::free(ptr, size)
    }

    /// Indicates if the platform can release a part of memory
    fn can_release_part(flags: u32) -> bool {
        sys::can_release_part(flags)
    }

    /// Allocates a memory region of zeros
    fn allocates_zeros() -> bool {
        sys::allocates_zeros()
    }

    /// Returns the page size
    fn page_size() -> usize {
        sys::page_size()
    }
}

/// A platform interface for platforms that support the global lock
#[cfg(feature = "global")]
pub trait GlobalSystem: System {
    /// Acquires the global lock
    fn acquire_global_lock();

    /// Releases the global lock
    fn release_global_lock();
}

/// Struct to implement the System trait
#[cfg(any(target_arch = "wasm32", target_os = "macos", target_env = "sgx"))]
pub struct Platform;
#[cfg(any(target_arch = "wasm32", target_os = "macos", target_env = "sgx"))]
impl System for Platform {}
#[cfg(feature = "global")]
#[cfg(any(target_arch = "wasm32", target_os = "macos", target_env = "sgx"))]
impl GlobalSystem for Platform {
    fn acquire_global_lock() {
        sys::acquire_global_lock()
    }

    fn release_global_lock() {
        sys::release_global_lock()
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "macos", target_env = "sgx")))]
/// A platform interface
pub trait System {
    /// Allocates a memory region of `size` bytes
    unsafe fn alloc(size: usize) -> (*mut u8, usize, u32);

    /// Remaps a memory region
    unsafe fn remap(ptr: *mut u8, oldsize: usize, newsize: usize, can_move: bool) -> *mut u8;

    /// Frees a part of a memory region
    unsafe fn free_part(ptr: *mut u8, oldsize: usize, newsize: usize) -> bool;

    /// Frees an entire memory region
    unsafe fn free(ptr: *mut u8, size: usize) -> bool;

    /// Indicates if the platform can release a part of memory
    fn can_release_part(flags: u32) -> bool;

    /// Allocates a memory region of zeros
    fn allocates_zeros() -> bool;

    /// Returns the page size
    fn page_size() -> usize;
}

/// An allocator instance
///
/// Instances of this type are used to allocate blocks of memory. For best
/// results only use one of these. Currently doesn't implement `Drop` to release
/// lingering memory back to the OS. That may happen eventually though!
pub struct Dlmalloc<S>(dlmalloc::Dlmalloc<S>);

/// Constant initializer for `Dlmalloc` structure.
pub const DLMALLOC_INIT: Dlmalloc<Platform> = Dlmalloc::new();

#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
mod sys;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod sys;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod sys;

#[cfg(target_os = "linux")]
pub use sys::Platform;

#[cfg(target_env = "sgx")]
#[path = "sgx.rs"]
mod sys;

impl<S> Dlmalloc<S> {
    /// Creates a new instance of an allocator
    pub const fn new() -> Dlmalloc<S> {
        Dlmalloc(dlmalloc::Dlmalloc::init())
    }
}

impl<S: System> Dlmalloc<S> {
    /// Allocates `size` bytes with `align` align.
    ///
    /// Returns a null pointer if allocation fails. Returns a valid pointer
    /// otherwise.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::alloc`
    /// method contracts.
    #[inline]
    pub unsafe fn malloc(&mut self, size: usize, align: usize) -> *mut u8 {
        if align <= self.0.malloc_alignment() {
            self.0.malloc(size)
        } else {
            self.0.memalign(align, size)
        }
    }

    /// Same as `malloc`, except if the allocation succeeds it's guaranteed to
    /// point to `size` bytes of zeros.
    #[inline]
    pub unsafe fn calloc(&mut self, size: usize, align: usize) -> *mut u8 {
        let ptr = self.malloc(size, align);
        if !ptr.is_null() && self.0.calloc_must_clear(ptr) {
            ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }

    /// Deallocates a `ptr` with `size` and `align` as the previous request used
    /// to allocate it.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::dealloc`
    /// method contracts.
    #[inline]
    pub unsafe fn free(&mut self, ptr: *mut u8, size: usize, align: usize) {
        drop((size, align));
        self.0.free(ptr)
    }

    /// Reallocates `ptr`, a previous allocation with `old_size` and
    /// `old_align`, to have `new_size` and the same alignment as before.
    ///
    /// Returns a null pointer if the memory couldn't be reallocated, but `ptr`
    /// is still valid. Returns a valid pointer and frees `ptr` if the request
    /// is satisfied.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::realloc`
    /// method contracts.
    #[inline]
    pub unsafe fn realloc(
        &mut self,
        ptr: *mut u8,
        old_size: usize,
        old_align: usize,
        new_size: usize,
    ) -> *mut u8 {
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
unsafe impl AllocRef for Dlmalloc {
    #[inline]
    fn alloc(&mut self, layout: Layout) -> Result<ptr::NonNull<[u8]>, AllocErr> {
        unsafe {
            let ptr = <Dlmalloc>::malloc(self, layout.size(), layout.align());
            let ptr = ptr::slice_from_raw_parts(ptr, layout.size());
            ptr::NonNull::new(ptr as _).ok_or(AllocErr)
        }
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: ptr::NonNull<u8>, layout: Layout) {
        <Dlmalloc>::free(self, ptr.as_ptr(), layout.size(), layout.align())
    }

    #[inline]
    fn alloc_zeroed(&mut self, layout: Layout) -> Result<ptr::NonNull<[u8]>, AllocErr> {
        unsafe {
            let ptr = <Dlmalloc>::calloc(self, layout.size(), layout.align());
            let ptr = ptr::slice_from_raw_parts(ptr, layout.size());
            ptr::NonNull::new(ptr as _).ok_or(AllocErr)
        }
    }
}
