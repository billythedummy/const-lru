//! TODO: seems like miri hangs because BigConstLru is too big
//! (works for smaller values of CAP).
//! Figure out how to re-enable miri for this and remove all the `#[cfg_attr(miri, ignore)]`s

use std::alloc::{alloc, Layout};

use const_lru::ConstLru;

// ~400 MB
type BigConstLru = ConstLru<usize, usize, 10_000_000>;
const BIG_CONST_LRU_LAYOUT: Layout = Layout::new::<BigConstLru>();

/*
/// Should fail with
/// `thread 'check_big_const_lru_stack_overflows' has overflowed its stack
/// fatal runtime error: stack overflow`
#[test]
fn check_big_const_lru_stack_overflows() {
    let _b = BigConstLru::new();
}
*/

fn boxed_big_const_lru() -> Box<BigConstLru> {
    unsafe {
        let ptr = alloc(BIG_CONST_LRU_LAYOUT) as *mut BigConstLru;
        ConstLru::init_at_alloc(ptr);
        Box::from_raw(ptr)
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn heap_alloc_doesnt_stack_overflow() {
    let mut c = boxed_big_const_lru();
    assert!(c.insert(1, 2).is_none());
}

#[test]
#[cfg_attr(miri, ignore)]
fn clear_doesnt_stack_overflow() {
    let mut c = boxed_big_const_lru();
    c.clear();
    assert!(c.insert(1, 2).is_none());
}

#[test]
#[cfg_attr(miri, ignore)]
fn clone_to_alloc_doesnt_stack_overflow() {
    let c = boxed_big_const_lru();
    let mut cloned = unsafe {
        let new_alloc_ptr = alloc(BIG_CONST_LRU_LAYOUT) as *mut BigConstLru;
        c.clone_to_alloc(new_alloc_ptr);
        Box::from_raw(new_alloc_ptr)
    };
    assert!(cloned.insert(1, 2).is_none());
}
