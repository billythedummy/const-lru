use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

/// A view into an vacant entry in a ConstLru. It is part of the Entry enum.
#[derive(Debug)]
pub struct VacantEntry<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    const_lru: &'a mut ConstLru<K, V, CAP, I>,
    key: K,
    insert_bs_i: I,
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> VacantEntry<'a, K, V, CAP, I> {
    pub(crate) fn new(const_lru: &'a mut ConstLru<K, V, CAP, I>, key: K, insert_bs_i: I) -> Self {
        Self {
            const_lru,
            key,
            insert_bs_i,
        }
    }

    /// Take ownership of the key.
    pub fn into_key(self) -> K {
        self.key
    }

    /// Gets a reference to the key that would be used when inserting a value through the `VacantEntry`.
    pub fn key(&self) -> &K {
        &self.key
    }
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> VacantEntry<'a, K, V, CAP, I> {
    /// Sets the value of the entry with the `VacantEntry`’s key, and returns:
    /// - a mutable reference to the new value
    /// - LRU evicted entry, if ConstLru is full
    pub fn insert(self, v: V) -> (&'a mut V, Option<(K, V)>) {
        let (i, opt) = if self.const_lru.is_full() {
            let (i, (old_k, old_v)) =
                self.const_lru
                    .insert_evict_lru(self.insert_bs_i, self.key, v);
            (i, Some((old_k, old_v)))
        } else {
            let i = self
                .const_lru
                .insert_alloc_new(self.insert_bs_i, self.key, v);
            (i, None)
        };
        (self.const_lru.get_mut_by_index(i), opt)
    }
}
