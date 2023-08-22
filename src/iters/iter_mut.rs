use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::double_ended_iter_cursors::DoubleEndedIterCursors;

/// Iterates through the keys and mutable values of the `ConstLru` from most-recently-used to least-recently-used
///
/// Does not change the LRU order of the elements.
pub struct IterMut<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    cursors: DoubleEndedIterCursors<I, CAP>,
    const_lru: &'a mut ConstLru<K, V, CAP, I>,
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> IterMut<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a mut ConstLru<K, V, CAP, I>) -> Self {
        let cursors = DoubleEndedIterCursors::new(const_lru);
        Self { cursors, const_lru }
    }

    fn get_entry_mut(&mut self, i: usize) -> (&'a K, &'a mut V) {
        // TODO: double check unsafes
        let key_ptr = unsafe { self.const_lru.keys[i].assume_init_ref() } as *const _;
        let key: &'a K = unsafe { &*key_ptr };
        let val_ptr = unsafe { self.const_lru.values[i].assume_init_mut() } as *mut _;
        let val: &'a mut V = unsafe { &mut *val_ptr };
        (key, val)
    }
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> Iterator for IterMut<'a, K, V, CAP, I> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        // consume then increment
        let i = self.cursors.get_from_head_idx();
        self.cursors.advance_from_head(self.const_lru);
        Some(self.get_entry_mut(i))
    }

    // TODO: look into https://doc.rust-lang.org/std/iter/trait.TrustedLen.html
    // and consider adding a `seen` field to implement it
    // when it lands in stable
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(CAP))
    }
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterMut<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        // decrement then consume
        self.cursors.retreat_from_tail(self.const_lru);
        let i = self.cursors.get_from_tail_idx();
        Some(self.get_entry_mut(i))
    }
}
