use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::in_order_bst::{DoubleEndedInOrderTraverser, InOrderTraversalState};

/// Iterates through the keys and mutable values of the `ConstLru` in the keys' sorted order
///
/// Does not change the LRU order of the elements.
pub struct IterKeyOrderMut<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    traverser: DoubleEndedInOrderTraverser<I>,
    const_lru: &'a mut ConstLru<K, V, CAP, I>,
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> IterKeyOrderMut<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a mut ConstLru<K, V, CAP, I>) -> Self {
        Self {
            traverser: DoubleEndedInOrderTraverser::new(const_lru),
            const_lru,
        }
    }

    fn get_entry_mut(&mut self, index: I) -> (&'a K, &'a mut V) {
        let i = index.to_usize().unwrap();
        // TODO: double check unsafes
        let key_ptr = unsafe { self.const_lru.keys[i].assume_init_ref() } as *const _;
        let key: &'a K = unsafe { &*key_ptr };
        let val_ptr = unsafe { self.const_lru.values[i].assume_init_mut() } as *mut _;
        let val: &'a mut V = unsafe { &mut *val_ptr };
        (key, val)
    }
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> Iterator
    for IterKeyOrderMut<'a, K, V, CAP, I>
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.traverser.has_ended() {
            return None;
        }
        // consume then increment
        // assert!(self.traverser.from_smallest().state() == InOrderTraversalState::Left);
        let res = self.get_entry_mut(self.traverser.get_from_smallest_current());
        self.traverser.advance_from_smallest(self.const_lru);
        while self.traverser.get_from_smallest_state() != InOrderTraversalState::Left {
            if self.traverser.has_ended() {
                break;
            }
            self.traverser.advance_from_smallest(self.const_lru);
        }
        Some(res)
    }

    // TODO: look into https://doc.rust-lang.org/std/iter/trait.TrustedLen.html when it lands in stable
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(CAP))
    }
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterKeyOrderMut<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // decrement then consume
        while self.traverser.get_from_largest_state() != InOrderTraversalState::This {
            if self.traverser.has_ended() {
                return None;
            }
            self.traverser.retreat_from_largest(self.const_lru);
        }
        let res = self.get_entry_mut(self.traverser.get_from_largest_current());
        self.traverser.retreat_from_largest(self.const_lru);
        Some(res)
    }
}
