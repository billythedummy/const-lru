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

    fn get_entry(&mut self, bs_i: I) -> (&'a K, &'a V) {
        let i = self.const_lru.bs_index[bs_i.to_usize().unwrap()]
            .to_usize()
            .unwrap();
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
