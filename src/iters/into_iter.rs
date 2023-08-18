use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::double_ended_iter_cursors::DoubleEndedIterCursors;

/// Iterates through the keys and values of the ConstLru from most-recently-used to least-recently-used
///
/// Does not change the LRU order of the elements.
pub struct IntoIter<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> {
    cursors: DoubleEndedIterCursors<I, CAP>,
    const_lru: ConstLru<K, V, CAP, I>,
}

impl<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> IntoIter<K, V, CAP, I> {
    pub fn new(const_lru: ConstLru<K, V, CAP, I>) -> Self {
        let cursors = DoubleEndedIterCursors::new(&const_lru);
        Self { cursors, const_lru }
    }

    fn get_entry(&mut self, i: usize) -> (K, V) {
        let key = unsafe { self.const_lru.keys[i].assume_init_read() };
        let val = unsafe { self.const_lru.values[i].assume_init_read() };
        (key, val)
    }
}

impl<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> Iterator for IntoIter<K, V, CAP, I> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        let i = self.cursors.get_from_head_idx();
        self.cursors.advance_from_head(&self.const_lru);
        // get_entry copies out (k, v),
        // we need to truncate the const_lru so that they dont get dropped again
        // when const_lru drops
        self.const_lru.head = self.cursors.get_from_head();
        self.const_lru.len = self.const_lru.len - I::one();
        Some(self.get_entry(i))
    }
}

impl<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IntoIter<K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        let i = self.cursors.get_from_tail_idx();
        self.cursors.retreat_from_tail(&self.const_lru);
        // get_entry copies out (k, v),
        // we need to truncate the const_lru so that they dont get dropped again
        // when const_lru drops
        self.const_lru.tail = self.cursors.get_from_tail();
        self.const_lru.len = self.const_lru.len - I::one();
        Some(self.get_entry(i))
    }
}
