use arbitrary::{Result, Unstructured};
use dlmalloc::Dlmalloc;
use std::cmp;

const MAX_ALLOCATED: usize = 100 << 20; // 100 MB

pub fn run(u: &mut Unstructured<'_>) -> Result<()> {
    let mut a = Dlmalloc::new();
    let mut ptrs = Vec::new();
    let mut allocated = 0;
    unsafe {
        while u.arbitrary()? {
            // If there are pointers to free then have a chance of deallocating
            // a pointer. Try not to deallocate things until there's a "large"
            // working set but afterwards give it a 50/50 chance of allocating
            // or deallocating.
            let free = match ptrs.len() {
                0 => false,
                0..=10_000 => u.ratio(1, 3)?,
                _ => u.arbitrary()?,
            };
            if free {
                let idx = u.choose_index(ptrs.len())?;
                let (ptr, size, align) = ptrs.swap_remove(idx);
                allocated -= size;
                a.free(ptr, size, align);
                continue;
            }

            // 1/100 chance of reallocating a pointer to a different size.
            if ptrs.len() > 0 && u.ratio(1, 100)? {
                let idx = u.choose_index(ptrs.len())?;
                let (ptr, size, align) = ptrs.swap_remove(idx);

                // Arbitrarily choose whether to make this allocation either
                // twice as large or half as small.
                let new_size = if u.arbitrary()? {
                    u.int_in_range(size..=size * 2)?
                } else if size > 10 {
                    u.int_in_range(size / 2..=size)?
                } else {
                    continue;
                };
                if allocated + new_size - size > MAX_ALLOCATED {
                    ptrs.push((ptr, size, align));
                    continue;
                }
                allocated -= size;
                allocated += new_size;

                // Perform the `realloc` and assert that all bytes were copied.
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

            // Aribtrarily choose a size to allocate as well as an alignment.
            // Enable small sizes with standard alignment happening a fair bit.
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

            if size + allocated > MAX_ALLOCATED {
                continue;
            }
            allocated += size;

            // Choose arbitrarily between a zero-allocated chunk and a normal
            // allocated chunk.
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

        // Deallocate everythign when we're done.
        for (ptr, size, align) in ptrs {
            a.free(ptr, size, align);
        }

        a.destroy();
    }

    Ok(())
}
