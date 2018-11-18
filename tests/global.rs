#![cfg(all(feature = "allocator-api", feature = "global"))]
#![feature(global_allocator)]

extern crate dlmalloc;

use std::collections::HashMap;
use std::thread;

#[global_allocator]
static A: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[test]
fn foo() {
    println!("hello");
}

#[test]
fn map() {
    let mut m = HashMap::new();
    m.insert(1, 2);
    m.insert(5, 3);
    drop(m);
}

#[test]
fn strings() {
    format!("foo, bar, {}", "baz");
}

#[test]
fn threads() {
    assert!(thread::spawn(|| panic!()).join().is_err());
}
