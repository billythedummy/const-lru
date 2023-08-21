use core::mem::MaybeUninit;

use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::double_ended_iter_cursors::DoubleEndedIterCursors;

/// Iterates through the keys and values of the ConstLru from most-recently-used to least-recently-used
///
/// Does not change the LRU order of the elements.
///
/// Only used to implement `Drop` for ConstLru
pub struct IterMaybeUninit<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    cursors: DoubleEndedIterCursors<I, CAP>,
    const_lru: &'a mut ConstLru<K, V, CAP, I>,
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> IterMaybeUninit<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a mut ConstLru<K, V, CAP, I>) -> Self {
        let cursors = DoubleEndedIterCursors::new(const_lru);
        Self { cursors, const_lru }
    }

    fn get_entry_mut(&mut self, i: usize) -> (&'a mut MaybeUninit<K>, &'a mut MaybeUninit<V>) {
        let key_ptr = &mut self.const_lru.keys[i] as *mut _;
        let key: &'a mut MaybeUninit<K> = unsafe { &mut *key_ptr };
        let val_ptr = &mut self.const_lru.values[i] as *mut _;
        let val: &'a mut MaybeUninit<V> = unsafe { &mut *val_ptr };
        (key, val)
    }
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> Iterator
    for IterMaybeUninit<'a, K, V, CAP, I>
{
    type Item = (&'a mut MaybeUninit<K>, &'a mut MaybeUninit<V>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursors.has_ended() {
            return None;
        }
        let i = self.cursors.get_from_head_idx();
        self.cursors.advance_from_head(self.const_lru);
        Some(self.get_entry_mut(i))
    }

    // TODO: look into https://doc.rust-lang.org/std/iter/trait.TrustedLen.html when it lands in stable
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(CAP))
    }
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterMaybeUninit<'a, K, V, CAP, I>
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
