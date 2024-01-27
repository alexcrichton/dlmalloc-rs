#![no_main]

use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let _ = dlmalloc_fuzz::run(&mut Unstructured::new(bytes));
});
