fn main() {
    build::main();
}

#[cfg(not(feature = "c"))]
mod build {
    pub fn main() {}
}

#[cfg(feature = "c")]
mod build {
    extern crate cc;

    use std::env;

    pub fn main() {
        let mut cfg = cc::Build::new();
        let target = env::var("TARGET").unwrap();
        if !target.contains("windows") {
            cfg.flag("-fvisibility=hidden");
        }
        cfg.file("src/dlmalloc.c");
        cfg.define("USE_LOCKS", None);
        cfg.compile("dlmalloc");
    }
}
