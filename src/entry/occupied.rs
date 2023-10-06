use num_traits::{PrimInt, Unsigned};

use crate::ConstLru;

/// A view into an occupied entry in a ConstLru. It is part of the Entry enum.
#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    const_lru: &'a mut ConstLru<K, V, CAP, I>,
    key: K,
    index: I,
    bs_i: I,
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> OccupiedEntry<'a, K, V, CAP, I> {
    pub(crate) fn new(
        const_lru: &'a mut ConstLru<K, V, CAP, I>,
        key: K,
        (index, bs_i): (I, I),
    ) -> Self {
        Self {
            const_lru,
            key,
            index,
            bs_i,
        }
    }

    /// Gets a reference to the value in the entry and moves entry to most recently used slot
    ///
    /// To not update to most-recently-used, use [`Self::get_untouched`]
    pub fn get(&mut self) -> &V {
        self.const_lru.move_to_head(self.index);
        self.const_lru.get_by_index(self.index)
    }

    /// Gets a reference to the value in the entry
    pub fn get_untouched(&self) -> &V {
        self.const_lru.get_by_index(self.index)
    }

    /// Gets a mutable reference to the value in the entry and moves entry to most recently used slot
    ///
    /// To not update to most-recently-used, use [`Self::get_mut_untouched`]
    pub fn get_mut(&mut self) -> &mut V {
        self.const_lru.move_to_head(self.index);
        self.const_lru.get_mut_by_index(self.index)
    }

    /// Gets a mutable reference to the value in the entry
    pub fn get_mut_untouched(&mut self) -> &mut V {
        self.const_lru.get_mut_by_index(self.index)
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, v: V) -> V {
        self.const_lru.insert_replace_value(self.index, v)
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry with a lifetime bound to the ConstLru itself.
    /// Also moves the entry to most recently used slot
    ///
    /// To not update to most-recently-used, use [`Self::into_mut_untouched`]
    pub fn into_mut(self) -> &'a mut V {
        self.const_lru.move_to_head(self.index);
        self.const_lru.get_mut_by_index(self.index)
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry with a lifetime bound to the ConstLru itself.
    pub fn into_mut_untouched(self) -> &'a mut V {
        self.const_lru.get_mut_by_index(self.index)
    }

    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Takes the value out of the entry, and returns it.
    pub fn remove(self) -> V {
        self.const_lru.remove_by_index((self.index, self.bs_i)).1
    }

    /// Take the ownership of the key and value from the ConstLru
    pub fn remove_entry(self) -> (K, V) {
        self.const_lru.remove_by_index((self.index, self.bs_i))
    }
}
