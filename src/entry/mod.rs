use num_traits::{PrimInt, Unsigned};

mod occupied;
mod vacant;

pub use occupied::*;
pub use vacant::*;

use crate::ConstLru;

/// A view into a single entry in a ConstLru, which may either be vacant or occupied.
#[derive(Debug)]
pub enum Entry<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> {
    Occupied(OccupiedEntry<'a, K, V, CAP, I>),
    Vacant(VacantEntry<'a, K, V, CAP, I>),
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> Entry<'a, K, V, CAP, I> {
    pub(crate) fn new(const_lru: &'a mut ConstLru<K, V, CAP, I>, k: K) -> Self {
        if CAP == 0 {
            panic!("Entry API only works for CAP > 0");
        }
        let insert_bs_i = match const_lru.get_index_of(&k) {
            Ok(tup) => return Self::Occupied(OccupiedEntry::new(const_lru, k, tup)),
            Err(i) => i,
        };
        Self::Vacant(VacantEntry::new(const_lru, k, insert_bs_i))
    }
}

impl<'a, K, V, const CAP: usize, I: PrimInt + Unsigned> Entry<'a, K, V, CAP, I> {
    /// Returns a reference to this entry’s key.
    pub fn key(&self) -> &K {
        match self {
            Self::Occupied(e) => e.key(),
            Self::Vacant(e) => e.key(),
        }
    }
}

impl<'a, K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> Entry<'a, K, V, CAP, I> {
    /// Ensures a value is in the entry by inserting the result of the default function if empty, and returns a mutable reference to the value in the entry.
    ///
    /// Also moves the entry to most-recently-used position if previously existing
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(e) => e.into_mut(),
            Self::Vacant(e) => e.insert(default()).0,
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function .
    /// This method allows for generating key-derived values for insertion by providing the default function a reference to the key
    /// that was moved during the .entry(key) method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is unnecessary, unlike with `.or_insert_with(|| ... )`
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(e) => e.into_mut(),
            Self::Vacant(e) => {
                let v = default(e.key());
                e.insert(v).0
            }
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and returns a mutable reference to the value in the entry.
    ///
    /// Also moves the entry to most-recently-used position if previously existing
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Self::Occupied(e) => e.into_mut(),
            Self::Vacant(e) => e.insert(default).0,
        }
    }
}

impl<'a, K: Ord, V: Default, const CAP: usize, I: PrimInt + Unsigned> Entry<'a, K, V, CAP, I> {
    /// Ensures a value is in the entry by inserting the default value if empty, and returns a mutable reference to the value in the entry.
    ///
    /// Also moves the entry to most-recently-used position if previously existing
    pub fn or_default(self) -> &'a mut V {
        self.or_insert(V::default())
    }
}
