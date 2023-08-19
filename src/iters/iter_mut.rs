use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::double_ended_iter_cursors::DoubleEndedIterCursors;

/// Iterates through the keys and mutable values of the ConstLru from most-recently-used to least-recently-used
///
/// Does not change the LRU order of the elements.
pub struct IterMut<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> {
    cursors: DoubleEndedIterCursors<I, CAP>,
    const_lru: &'a mut ConstLru<K, V, CAP, I>,
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> IterMut<'a, K, V, CAP, I> {
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

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> Iterator for IterMut<'a, K, V, CAP, I> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        let i = self.cursors.get_from_head_idx();
        self.cursors.advance_from_head(self.const_lru);
        Some(self.get_entry_mut(i))
    }
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterMut<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        let i = self.cursors.get_from_tail_idx();
        self.cursors.retreat_from_tail(self.const_lru);
        Some(self.get_entry_mut(i))
    }
}

/// mut iterator that also returns the index of the current element
///
/// Used for internal implementation
pub struct IterMutIndexed<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned>(
    IterMut<'a, K, V, CAP, I>,
);

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> IterMutIndexed<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a mut ConstLru<K, V, CAP, I>) -> Self {
        Self(IterMut::new(const_lru))
    }
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> Iterator
    for IterMutIndexed<'a, K, V, CAP, I>
{
    type Item = (I, &'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.0.cursors.get_from_head();
        // next() modifies cursors, so extract index first
        self.0.next().map(|(k, v)| (i, k, v))
    }
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterMutIndexed<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let i = self.0.cursors.get_from_tail();
        // next_back() modifies cursors, so extract index first
        self.0.next_back().map(|(k, v)| (i, k, v))
    }
}
