use arbitrary::Unstructured;
use dlmalloc::Dlmalloc;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

#[test]
fn smoke() {
    let mut a = Dlmalloc::new();
    unsafe {
        let ptr = a.malloc(1, 1);
        assert!(!ptr.is_null());
        *ptr = 9;
        assert_eq!(*ptr, 9);
        a.free(ptr, 1, 1);

        let ptr = a.malloc(1, 1);
        assert!(!ptr.is_null());
        *ptr = 10;
        assert_eq!(*ptr, 10);
        a.free(ptr, 1, 1);
    }
}

#[test]
fn no_layout_round_trip() {
    let mut a = Dlmalloc::new();
    unsafe {
        let ptr = a.malloc_no_layout(32);
        assert!(!ptr.is_null());
        ptr.write_bytes(0xab, 32);
        assert_eq!(*ptr, 0xab);
        assert_eq!(*ptr.add(31), 0xab);

        let grown = a.realloc_no_layout(ptr, 128);
        assert!(!grown.is_null());
        for i in 0..32 {
            assert_eq!(*grown.add(i), 0xab);
        }
        grown.add(32).write_bytes(0xcd, 96);
        assert_eq!(*grown.add(127), 0xcd);

        let shrunk = a.realloc_no_layout(grown, 16);
        assert!(!shrunk.is_null());
        for i in 0..16 {
            assert_eq!(*shrunk.add(i), 0xab);
        }

        a.free_no_layout(shrunk);
    }
}

#[test]
fn no_layout_null_handling() {
    let mut a = Dlmalloc::new();
    unsafe {
        a.free_no_layout(core::ptr::null_mut());

        let ptr = a.realloc_no_layout(core::ptr::null_mut(), 64);
        assert!(!ptr.is_null());
        ptr.write_bytes(0x5a, 64);
        assert_eq!(*ptr, 0x5a);
        assert_eq!(*ptr.add(63), 0x5a);
        a.free_no_layout(ptr);
    }
}

#[test]
fn memalign_no_layout_round_trip() {
    let mut a = Dlmalloc::new();
    unsafe {
        for &align in &[1usize, 2, 8, 16, 32, 64, 256, 4096] {
            let ptr = a.memalign_no_layout(align, 96);
            assert!(!ptr.is_null(), "memalign_no_layout({align}, 96) failed");
            assert_eq!(
                (ptr as usize) & (align - 1),
                0,
                "ptr {ptr:p} not aligned to {align}"
            );
            ptr.write_bytes(0x77, 96);
            assert_eq!(*ptr, 0x77);
            assert_eq!(*ptr.add(95), 0x77);

            let grown = a.realloc_no_layout(ptr, 256);
            assert!(!grown.is_null());
            for i in 0..96 {
                assert_eq!(*grown.add(i), 0x77);
            }

            a.free_no_layout(grown);
        }
    }
}

#[path = "../fuzz/src/lib.rs"]
mod fuzz;

#[test]
fn stress() {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut buf = vec![0; 4096];
    let iters = if cfg!(miri) { 5 } else { 2000 };
    for _ in 0..iters {
        rng.fill_bytes(&mut buf);
        let mut u = Unstructured::new(&buf);
        let _ = fuzz::run(&mut u);
    }
}
