use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use const_lru::ConstLru;
use num_traits::{PrimInt, Unsigned};

use super::utils::boxed_const_lru;

pub trait Get<K, V> {
    fn get_by_key(&mut self, k: &K) -> Option<&V>;
}

impl<K: Eq + Hash, V, S: BuildHasher> Get<K, V> for HashMap<K, V, S> {
    fn get_by_key(&mut self, k: &K) -> Option<&V> {
        self.get(k)
    }
}

impl<K: Ord, V, const CAP: usize, I: Unsigned + PrimInt> Get<K, V> for ConstLru<K, V, CAP, I> {
    fn get_by_key(&mut self, k: &K) -> Option<&V> {
        self.get(k)
    }
}

pub trait Insert<K, V> {
    fn insert_no_ret(&mut self, k: K, v: V);
}

impl<K: Eq + Hash, V, S: BuildHasher> Insert<K, V> for HashMap<K, V, S> {
    fn insert_no_ret(&mut self, k: K, v: V) {
        self.insert(k, v);
    }
}

impl<K: Ord, V, const CAP: usize, I: Unsigned + PrimInt> Insert<K, V> for ConstLru<K, V, CAP, I> {
    fn insert_no_ret(&mut self, k: K, v: V) {
        self.insert(k, v);
    }
}

impl<K: Ord, V, const CAP: usize, I: Unsigned + PrimInt> Insert<K, V>
    for Box<ConstLru<K, V, CAP, I>>
{
    fn insert_no_ret(&mut self, k: K, v: V) {
        self.insert(k, v);
    }
}

pub trait Remove<K, V> {
    fn remove_by_key(&mut self, k: &K) -> Option<V>;
}

impl<K: Eq + Hash, V, S: BuildHasher> Remove<K, V> for HashMap<K, V, S> {
    fn remove_by_key(&mut self, k: &K) -> Option<V> {
        self.remove(k)
    }
}

impl<K: Ord, V, const CAP: usize, I: Unsigned + PrimInt> Remove<K, V> for ConstLru<K, V, CAP, I> {
    fn remove_by_key(&mut self, k: &K) -> Option<V> {
        self.remove(k)
    }
}

impl<K: Ord, V, const CAP: usize, I: Unsigned + PrimInt> Remove<K, V>
    for Box<ConstLru<K, V, CAP, I>>
{
    fn remove_by_key(&mut self, k: &K) -> Option<V> {
        self.remove(k)
    }
}

pub trait CreateNew {
    fn create_new() -> Self;
}

impl<K, V> CreateNew for HashMap<K, V> {
    fn create_new() -> Self {
        Self::new()
    }
}

impl<K, V, const CAP: usize, I: Unsigned + PrimInt> CreateNew for ConstLru<K, V, CAP, I> {
    fn create_new() -> Self {
        Self::new()
    }
}

impl<K, V, const CAP: usize, I: Unsigned + PrimInt> CreateNew for Box<ConstLru<K, V, CAP, I>> {
    fn create_new() -> Self {
        boxed_const_lru()
    }
}
