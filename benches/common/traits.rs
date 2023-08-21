use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

use const_lru::ConstLru;
use num_traits::{PrimInt, Unsigned};

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
