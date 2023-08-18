use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::double_ended_iter_cursors::DoubleEndedIterCursors;

/// Iterates through the keys and values of the ConstLru from most-recently-used to least-recently-used
///
/// Does not change the LRU order of the elements.
pub struct Iter<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> {
    cursors: DoubleEndedIterCursors<I, CAP>,
    const_lru: &'a ConstLru<K, V, CAP, I>,
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> Iter<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a ConstLru<K, V, CAP, I>) -> Self {
        let cursors = DoubleEndedIterCursors::new(const_lru);
        Self { cursors, const_lru }
    }

    pub fn cursors(&self) -> &DoubleEndedIterCursors<I, CAP> {
        &self.cursors
    }

    fn get_entry(&mut self, i: usize) -> (&'a K, &'a V) {
        let key = unsafe { self.const_lru.keys[i].assume_init_ref() };
        let val = unsafe { self.const_lru.values[i].assume_init_ref() };
        (key, val)
    }
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> Iterator for Iter<'a, K, V, CAP, I> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        let i = self.cursors.get_from_head_idx();
        self.cursors.advance_from_head(self.const_lru);
        Some(self.get_entry(i))
    }
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for Iter<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        let i = self.cursors.get_from_tail_idx();
        self.cursors.retreat_from_tail(self.const_lru);
        Some(self.get_entry(i))
    }
}
