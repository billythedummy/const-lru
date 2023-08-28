use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

/// Iterates through the keys and values of the `ConstLru` in the keys' sorted order
///
/// Does not change the LRU order of the elements.
pub struct IterKeyOrder<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    /// from_smallest_bsi == from_largest_bsi means ended
    from_smallest_bsi: I,
    from_largest_bsi: I,
    const_lru: &'a ConstLru<K, V, CAP, I>,
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> IterKeyOrder<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a ConstLru<K, V, CAP, I>) -> Self {
        Self {
            from_smallest_bsi: I::zero(),
            from_largest_bsi: const_lru.len(),
            const_lru,
        }
    }

    /// Assumes bs_i is in bounds
    /// returns const_lru.bs_index[bs_i]
    fn get_index(&self, bs_i: I) -> I {
        self.const_lru.bs_index[bs_i.to_usize().unwrap()]
    }

    /// Assumes bs_i is in bounds
    fn get_entry(&mut self, bs_i: I) -> (&'a K, &'a V) {
        let i = self.get_index(bs_i).to_usize().unwrap();
        let key = unsafe { self.const_lru.keys[i].assume_init_ref() };
        let val = unsafe { self.const_lru.values[i].assume_init_ref() };
        (key, val)
    }

    fn has_ended(&self) -> bool {
        self.from_smallest_bsi == self.from_largest_bsi
    }
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> Iterator
    for IterKeyOrder<'a, K, V, CAP, I>
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_ended() {
            return None;
        }
        // consume then increment
        let res = self.get_entry(self.from_smallest_bsi);
        self.from_smallest_bsi = self.from_smallest_bsi + I::one();
        Some(res)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = (self.from_largest_bsi - self.from_smallest_bsi)
            .to_usize()
            .unwrap();
        (l, Some(l))
    }
}

// TODO: look into https://doc.rust-lang.org/std/iter/trait.TrustedLen.html when it lands in stable
impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> ExactSizeIterator
    for IterKeyOrder<'a, K, V, CAP, I>
{
}

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

/// Iterator that also returns the index of the current element
///
/// Used for internal implementation, currently only used to impl clone()
pub struct IterIndexed<'a, K, V, const CAP: usize, I: PrimInt + Unsigned>(
    IterKeyOrder<'a, K, V, CAP, I>,
);

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> IterIndexed<'a, K, V, CAP, I> {
    pub fn new(const_lru: &'a ConstLru<K, V, CAP, I>) -> Self {
        Self(IterKeyOrder::new(const_lru))
    }
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> Iterator for IterIndexed<'a, K, V, CAP, I> {
    type Item = (I, &'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.has_ended() {
            return None;
        }
        // consume then increment
        // next() modifies from_smallest_bsi, so read index out first
        let i = self.0.get_index(self.0.from_smallest_bsi);
        let (k, v) = self.0.next().unwrap();
        Some((i, k, v))
    }

    // TODO: look into https://doc.rust-lang.org/std/iter/trait.TrustedLen.html when it lands in stable
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// TODO: look into https://doc.rust-lang.org/std/iter/trait.TrustedLen.html when it lands in stable
impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> ExactSizeIterator
    for IterIndexed<'a, K, V, CAP, I>
{
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> DoubleEndedIterator
    for IterIndexed<'a, K, V, CAP, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // decrement then consume
        self.0
            .next_back()
            .map(|(k, v)| (self.0.get_index(self.0.from_largest_bsi), k, v))
    }
}
