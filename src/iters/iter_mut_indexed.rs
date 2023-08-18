use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

use super::iter_mut::IterMut;

/// mut iterator that also returns the index of the current element
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
        let i = self.0.cursors().get_from_head();
        // next() modifies cursors, so extract index first
        self.0.next().map(|(k, v)| (i, k, v))
    }
}

impl<'a, K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterMutIndexed<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let i = self.0.cursors().get_from_tail();
        // next_back() modifies cursors, so extract index first
        self.0.next_back().map(|(k, v)| (i, k, v))
    }
}
