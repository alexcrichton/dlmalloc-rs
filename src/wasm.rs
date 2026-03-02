use crate::Allocator;
#[cfg(target_arch = "wasm32")]
use core::arch::wasm32 as wasm;
#[cfg(target_arch = "wasm64")]
use core::arch::wasm64 as wasm;
use core::ptr;
use core::sync::atomic::{AtomicU8, Ordering};

extern "C" {
    static __heap_base: u8;
    static __heap_end: u8;
}

const PREEXISTING_UNTRIED: u8 = 0;
const PREEXISTING_DONATED: u8 = 1;
const PREEXISTING_DISABLED: u8 = 2;

static PREEXISTING_STATE: AtomicU8 = AtomicU8::new(PREEXISTING_UNTRIED);

fn preexisting_chunk(size: usize, heap_base: usize, heap_end: usize) -> Option<(usize, usize)> {
    let start = heap_base;
    if start == 0 {
        return None;
    }

    if heap_end <= start {
        return None;
    }

    let len = heap_end - start;
    if len < size {
        return None;
    }

    Some((start, len))
}

fn try_donate_preexisting(
    state: &AtomicU8,
    chunk: Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    if state.load(Ordering::Relaxed) != PREEXISTING_UNTRIED {
        return None;
    }

    match chunk {
        Some(chunk) => {
            if state
                .compare_exchange(
                    PREEXISTING_UNTRIED,
                    PREEXISTING_DONATED,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                Some(chunk)
            } else {
                None
            }
        }
        None => {
            let _ = state.compare_exchange(
                PREEXISTING_UNTRIED,
                PREEXISTING_DISABLED,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            None
        }
    }
}

fn alloc_via_grow(size: usize, page_size: usize) -> (*mut u8, usize, u32) {
    let pages = size.div_ceil(page_size);
    let prev = wasm::memory_grow(0, pages);

    if prev == usize::max_value() {
        return (ptr::null_mut(), 0, 0);
    }

    let prev_page = prev * page_size;
    let base_ptr = prev_page as *mut u8;
    let size = pages * page_size;

    // Additionally check to see if we just allocated the final bit of the
    // address space. In such a situation it's not valid in Rust for a
    // pointer to actually wrap around to from the top of the address space
    // to 0, so it's not valid to allocate the entire region. Fake the last
    // few bytes as being un-allocated meaning that the actual size of this
    // allocation won't be page aligned, which should be handled by
    // dlmalloc.
    if prev_page.wrapping_add(size) == 0 {
        return (base_ptr, size - 16, 0);
    }

    (base_ptr, size, 0)
}

/// System setting for Wasm.
///
/// This is the default wasm allocator backend and only allocates by growing
/// linear memory with `memory.grow`.
pub struct System {
    _priv: (),
}

impl System {
    pub const fn new() -> System {
        System { _priv: () }
    }
}

unsafe impl Allocator for System {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        alloc_via_grow(size, self.page_size())
    }

    fn remap(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        ptr::null_mut()
    }

    fn free_part(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        false
    }

    fn free(&self, _ptr: *mut u8, _size: usize) -> bool {
        false
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        false
    }

    fn allocates_zeros(&self) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        64 * 1024
    }
}

/// Opt-in wasm allocator backend that can donate the pre-existing linear
/// memory region to dlmalloc once.
///
/// This allocator assumes the region between `__heap_base` and `__heap_end`
/// can be exclusively owned by dlmalloc. Only use this if no other runtime or
/// allocator in the module also expects to own that region.
pub struct PreexistingSystem {
    _priv: (),
}

impl PreexistingSystem {
    /// Creates a new opt-in preexisting-heap allocator backend.
    pub const fn new() -> PreexistingSystem {
        PreexistingSystem { _priv: () }
    }
}

unsafe impl Allocator for PreexistingSystem {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        let page_size = self.page_size();

        if size != 0 {
            let heap_base = unsafe { &__heap_base as *const u8 as usize };
            let heap_end = unsafe { &__heap_end as *const u8 as usize };

            let chunk = preexisting_chunk(size, heap_base, heap_end);
            if let Some((base, len)) = try_donate_preexisting(&PREEXISTING_STATE, chunk) {
                return (base as *mut u8, len, 0);
            }
        }

        alloc_via_grow(size, page_size)
    }

    fn remap(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        ptr::null_mut()
    }

    fn free_part(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        false
    }

    fn free(&self, _ptr: *mut u8, _size: usize) -> bool {
        false
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        false
    }

    fn allocates_zeros(&self) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        64 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::{
        preexisting_chunk, try_donate_preexisting, PREEXISTING_DISABLED, PREEXISTING_DONATED,
        PREEXISTING_UNTRIED,
    };
    use core::sync::atomic::{AtomicU8, Ordering};

    fn legacy_grow_only(
        size: usize,
        page_size: usize,
        grow_result: usize,
    ) -> Option<(usize, usize)> {
        let pages = size.div_ceil(page_size);
        if grow_result == usize::MAX {
            return None;
        }
        Some((grow_result * page_size, pages * page_size))
    }

    #[test]
    fn uses_preexisting_memory_when_growth_fails() {
        let page_size = 64 * 1024;
        let heap_base = page_size;
        let heap_end = page_size * 4;

        let chunk = preexisting_chunk(16, heap_base, heap_end).unwrap();
        let state = AtomicU8::new(PREEXISTING_UNTRIED);
        let new_behavior = try_donate_preexisting(&state, Some(chunk));
        let legacy_behavior = legacy_grow_only(16, page_size, usize::MAX);

        assert_eq!(new_behavior, Some((page_size, page_size * 3)));
        assert_eq!(legacy_behavior, None);
        assert_eq!(state.load(Ordering::Relaxed), PREEXISTING_DONATED);
    }

    #[test]
    fn uses_heap_end_as_upper_bound() {
        let page_size = 64 * 1024;
        let heap_base = page_size;
        let heap_end = page_size * 4;

        let chunk = preexisting_chunk(16, heap_base, heap_end).unwrap();
        assert_eq!(chunk, (page_size, page_size * 3));
    }

    #[test]
    fn one_chunk_or_never_disables_after_failure() {
        let page_size = 64 * 1024;
        let heap_base = page_size;
        let heap_end = page_size * 2;
        let state = AtomicU8::new(PREEXISTING_UNTRIED);

        let first = preexisting_chunk(page_size * 3, heap_base, heap_end);
        assert_eq!(first, None);
        assert_eq!(try_donate_preexisting(&state, first), None);
        assert_eq!(state.load(Ordering::Relaxed), PREEXISTING_DISABLED);

        let second = preexisting_chunk(16, heap_base, heap_end);
        assert_eq!(second, Some((page_size, page_size)));
        assert_eq!(try_donate_preexisting(&state, second), None);
    }

    #[test]
    fn one_chunk_donates_only_once() {
        let state = AtomicU8::new(PREEXISTING_UNTRIED);

        assert_eq!(
            try_donate_preexisting(&state, Some((64 * 1024, 128 * 1024))),
            Some((64 * 1024, 128 * 1024))
        );
        assert_eq!(state.load(Ordering::Relaxed), PREEXISTING_DONATED);
        assert_eq!(
            try_donate_preexisting(&state, Some((64 * 1024, 64 * 1024))),
            None
        );
    }

    #[test]
    fn rejects_zero_start() {
        let page_size = 64 * 1024;
        let heap_base = 0;
        let heap_end = page_size * 4;

        assert_eq!(preexisting_chunk(16, heap_base, heap_end), None);
    }
}

#[cfg(feature = "global")]
pub fn acquire_global_lock() {
    // single threaded, no need!
    assert!(!cfg!(target_feature = "atomics"));
}

#[cfg(feature = "global")]
pub fn release_global_lock() {
    // single threaded, no need!
    assert!(!cfg!(target_feature = "atomics"));
}

#[allow(missing_docs)]
#[cfg(feature = "global")]
pub unsafe fn enable_alloc_after_fork() {
    // single threaded, no need!
    assert!(!cfg!(target_feature = "atomics"));
}
