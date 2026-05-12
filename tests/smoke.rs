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

        a.free_no_layout(grown);
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
