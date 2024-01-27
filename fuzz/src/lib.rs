use arbitrary::{Result, Unstructured};
use dlmalloc::Dlmalloc;
use std::cmp;

pub fn run(u: &mut Unstructured<'_>) -> Result<()> {
    let mut a = Dlmalloc::new();
    let mut ptrs = Vec::new();
    unsafe {
        while u.arbitrary()? {
            let free =
                ptrs.len() > 0 && ((ptrs.len() < 10_000 && u.ratio(1, 3)?) || u.arbitrary()?);
            if free {
                let idx = u.choose_index(ptrs.len())?;
                let (ptr, size, align) = ptrs.swap_remove(idx);
                a.free(ptr, size, align);
                continue;
            }

            if ptrs.len() > 0 && u.ratio(1, 100)? {
                let idx = u.choose_index(ptrs.len())?;
                let (ptr, size, align) = ptrs.swap_remove(idx);
                let new_size = if u.arbitrary()? {
                    u.int_in_range(size..=size * 2)?
                } else if size > 10 {
                    u.int_in_range(size / 2..=size)?
                } else {
                    continue;
                };
                let mut tmp = Vec::new();
                for i in 0..cmp::min(size, new_size) {
                    tmp.push(*ptr.offset(i as isize));
                }
                let ptr = a.realloc(ptr, size, align, new_size);
                assert!(!ptr.is_null());
                for (i, byte) in tmp.iter().enumerate() {
                    assert_eq!(*byte, *ptr.offset(i as isize));
                }
                ptrs.push((ptr, new_size, align));
            }

            let size = if u.arbitrary()? {
                u.int_in_range(1..=128)?
            } else {
                u.int_in_range(1..=128 * 1024)?
            };
            let align = if u.ratio(1, 10)? {
                1 << u.int_in_range(3..=8)?
            } else {
                8
            };

            let zero = u.ratio(1, 50)?;
            let ptr = if zero {
                a.calloc(size, align)
            } else {
                a.malloc(size, align)
            };
            for i in 0..size {
                if zero {
                    assert_eq!(*ptr.offset(i as isize), 0);
                }
                *ptr.offset(i as isize) = 0xce;
            }
            ptrs.push((ptr, size, align));
        }

        for (ptr, size, align) in ptrs {
            a.free(ptr, size, align);
        }
    }

    Ok(())
}
