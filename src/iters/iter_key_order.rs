use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::in_order_bst::{DoubleEndedInOrderTraverser, InOrderTraversalState};

/// Iterates through the keys and values of the `ConstLru` in the keys' sorted order
///
/// Does not change the LRU order of the elements.
pub struct IterKeyOrder<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    traverser: DoubleEndedInOrderTraverser<I>,
    const_lru: &'a ConstLru<K, V, CAP, I>,
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> IterKeyOrder<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a ConstLru<K, V, CAP, I>) -> Self {
        Self {
            traverser: DoubleEndedInOrderTraverser::new(const_lru),
            const_lru,
        }
    }

    fn get_entry(&self, index: I) -> (&'a K, &'a V) {
        let i = index.to_usize().unwrap();
        let key = unsafe { self.const_lru.keys[i].assume_init_ref() };
        let val = unsafe { self.const_lru.values[i].assume_init_ref() };
        (key, val)
    }
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> Iterator
    for IterKeyOrder<'a, K, V, CAP, I>
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.traverser.has_ended() {
            return None;
        }
        // consume then increment
        // assert!(self.traverser.from_smallest().state() == InOrderTraversalState::Left);
        let res = self.get_entry(self.traverser.get_from_smallest_current());
        self.traverser.advance_from_smallest(self.const_lru);
        while self.traverser.get_from_smallest_state() != InOrderTraversalState::Left {
            if self.traverser.has_ended() {
                break;
            }
            self.traverser.advance_from_smallest(self.const_lru);
        }
        Some(res)
    }
}

/*
impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterKeyOrder<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.has_ended() {
            return None;
        }
        // decrement then consume
        self.from_largest_bsi = self.from_largest_bsi - I::one();
        let res = self.get_entry(self.from_largest_bsi);
        Some(res)
    }
}
*/
