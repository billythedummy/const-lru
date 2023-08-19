#![no_std]
#![doc = include_str!("../README.md")]

use core::borrow::Borrow;
use core::mem::MaybeUninit;
use num_traits::{PrimInt, Unsigned};

mod iters;

pub use iters::double_ended_iter_cursors::DoubleEndedIterCursors;
pub use iters::into_iter::IntoIter;
pub use iters::iter::Iter;
pub use iters::iter_mut::IterMut;

use iters::iter_indexed::IterIndexed;
use iters::iter_maybe_uninit::IterMaybeUninit;
use iters::iter_mut_indexed::IterMutIndexed;

/// Constant capacity key-addressed LRU cache.
///
/// Generics:
/// - `K`. Type of key. `Eq` is used to address entries.
/// - `V`. Type of value.
/// - `CAP`. Capacity of the cache. Must be > 0. All memory is allocated upfront.
/// - `I`. Type of the index used. Should be an unsigned primitive type smaller in bitwidth than `usize`.
///
/// Some implementation details:
/// - Fields are arranged in a struct-of-arrays format
#[derive(Debug)]
pub struct ConstLru<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned = usize> {
    len: I,

    /// head is index of most recently used
    ///
    /// can be any value if cache is empty
    head: I,

    /// tail is index of least recently used
    ///
    /// if cache is empty, tail is the first slot of unallocated memory / "free-list"
    /// else, next of the tail is the first slot of unallocated memory / "free-list"
    ///
    /// tail is always < CAP
    tail: I,

    /// disregard if value == CAP
    nexts: [I; CAP],

    /// disregard if value == CAP
    prevs: [I; CAP],

    keys: [MaybeUninit<K>; CAP],

    values: [MaybeUninit<V>; CAP],
}

impl<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> ConstLru<K, V, CAP, I> {
    /// Creates a new ConstLru
    ///
    /// panics if
    /// - CAP > I::MAX
    /// - I::MAX > usize::MAX
    /// - CAP == 0
    pub fn new() -> Self {
        let i_max = I::max_value()
            .to_usize()
            .unwrap_or_else(|| panic!("I::MAX > usize::MAX"));
        if CAP > i_max {
            panic!("CAP > I::MAX");
        }
        if CAP == 0 {
            panic!("CAP == 0");
        }

        let cap = I::from(CAP).unwrap();

        // [1, 2, ..., cap-1, cap]
        let mut nexts = [cap; CAP];
        for (i, next) in nexts.iter_mut().enumerate().take(CAP - 1) {
            *next = I::from(i + 1).unwrap();
        }

        // [cap, 0, 1, ..., cap-2]
        let mut prevs = [cap; CAP];
        for (i, prev) in prevs.iter_mut().enumerate().skip(1) {
            *prev = I::from(i - 1).unwrap();
        }

        Self {
            len: I::zero(),
            head: cap,
            tail: I::zero(),
            nexts,
            prevs,
            keys: unsafe { MaybeUninit::uninit().assume_init() },
            values: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    /// Inserts a key-value pair into the map. The entry is moved to the most-recently-used slot
    ///
    /// If the map did not have this key present and is not full, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    /// The key is not updated, though; this matters for types that can be == without being identical.
    ///
    /// If the map is full, the least-recently used key-value pair is evicted and returned.
    pub fn insert(&mut self, k: K, v: V) -> Option<InsertReplaced<K, V>> {
        for (i, old_k, old_v) in IterMutIndexed::new(self) {
            if *old_k == k {
                let old_v_out = core::mem::replace(old_v, v);
                self.move_to_head(i);
                return Some(InsertReplaced::OldValue(old_v_out));
            }
        }
        if self.is_full() {
            // N > 0, tail must be valid
            let t = self.tail.to_usize().unwrap();
            let evicted_k = unsafe { self.keys[t].assume_init_read() };
            let evicted_v = unsafe { self.values[t].assume_init_read() };
            self.keys[t].write(k);
            self.values[t].write(v);
            self.move_to_head(self.tail);
            return Some(InsertReplaced::LruEvicted(evicted_k, evicted_v));
        }
        // else alloc new node
        let free_index = if self.is_empty() {
            self.head = self.tail;
            self.tail
        } else {
            self.nexts[self.tail.to_usize().unwrap()]
        };
        self.tail = free_index;
        let f = free_index.to_usize().unwrap();
        self.keys[f].write(k);
        self.values[f].write(v);
        self.len = self.len + I::one();

        self.move_to_head(self.tail);
        None
    }

    pub fn remove<Q: Eq>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let index = self.get_index_of(k)?;
        let i = index.to_usize().unwrap();

        unsafe {
            self.keys[i].assume_init_drop();
        }
        let val = unsafe { self.values[i].assume_init_read() };

        // if len == 1, correct links are already in place
        if self.len() > I::one() {
            // len > 1
            // move to front of free list
            self.unlink_node(index);
            let t = self.tail.to_usize().unwrap();
            let first_free = self.nexts[t];

            if first_free < self.cap() {
                self.prevs[first_free.to_usize().unwrap()] = index;
            }
            self.nexts[i] = first_free;

            self.prevs[i] = self.tail;
            self.nexts[t] = index;
        }

        self.len = self.len - I::one();
        Some(val)
    }

    /// private helper fn.
    ///
    /// Unlinks the node at `index` from the doubly-linked list,
    /// patching its previous and next nodes, as well as self.head and self.tail if required.
    ///
    /// Can be used on both valid and invalid nodes.
    ///
    /// When this fn returns, `index`'s next and prev should be treated as invalid
    ///
    /// `self.head` and `self.tail` are not modified if only 1 elem in list
    ///
    /// Requirements:
    /// - index < CAP
    fn unlink_node(&mut self, index: I) {
        let i = index.to_usize().unwrap();
        let next = self.nexts[i];
        let prev = self.prevs[i];

        // index.next.prev = index.prev
        if next != self.cap() {
            self.prevs[next.to_usize().unwrap()] = prev;
        }

        // index.prev.next = index.next
        if prev != self.cap() {
            self.nexts[prev.to_usize().unwrap()] = next;
        }

        let is_one_elem_list = self.head == self.tail;

        if self.head == index && !is_one_elem_list {
            self.head = next;
        }

        if self.tail == index && !is_one_elem_list {
            self.tail = prev;
        }
    }

    /// private helper fn.
    ///
    /// Moves the element at index to the most-recently-used position.
    ///
    /// Requirements:
    /// - !self.is_empty()
    /// - index must be that of a valid node
    fn move_to_head(&mut self, index: I) {
        if self.head == index {
            return;
        }

        self.unlink_node(index);
        let i = index.to_usize().unwrap();

        // since self.head != index
        // and index is valid,
        // head must be valid
        let head = self.head;
        self.prevs[i] = self.cap();
        self.nexts[i] = head;

        self.prevs[head.to_usize().unwrap()] = index;

        self.head = index;
    }

    /// Gets reference to a value and moves entry to most-recently-used slot.
    ///
    /// To not update to most-recently-used, use [`get_untouched`]
    pub fn get<Q: Eq>(&mut self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let index = self.get_index_of(k)?;
        self.move_to_head(index);
        Some(unsafe { self.values[index.to_usize().unwrap()].assume_init_ref() })
    }

    /// Gets mut reference to a value and moves entry to most-recently-used slot.
    ///
    /// To not update to most-recently-used, use [`get_mut_untouched`]
    pub fn get_mut<Q: Eq>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let index = self.get_index_of(k)?;
        self.move_to_head(index);
        Some(unsafe { self.values[index.to_usize().unwrap()].assume_init_mut() })
    }

    fn get_index_of<Q: Eq>(&self, k: &Q) -> Option<I>
    where
        K: Borrow<Q>,
    {
        for (index, in_k, _in_v) in IterIndexed::new(self) {
            if in_k.borrow() == k {
                return Some(index);
            }
        }
        None
    }

    /// Get reference to value without updating the entry to most-recently-used slot
    ///
    /// To update to most-recently-used, use [`get`]
    pub fn get_untouched<Q: Eq>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        for (in_k, in_v) in self.iter() {
            if in_k.borrow() == k {
                return Some(in_v);
            }
        }
        None
    }

    /// Get mut reference to value without updating the entry to most-recently-used slot
    ///
    /// To update to most-recently-used, use [`get_mut`]
    pub fn get_mut_untouched<Q: Eq>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        for (in_k, in_v) in self.iter_mut() {
            if in_k.borrow() == k {
                return Some(in_v);
            }
        }
        None
    }

    /// Creates an iterator that iterates through the keys and values of the ConstLru from most-recently-used to least-recently-used
    ///
    /// Does not change the LRU order of the elements.
    ///
    /// Double-ended: reversing iterates from least-recently-used to most-recently-used
    pub fn iter(&self) -> Iter<K, V, CAP, I> {
        Iter::new(self)
    }

    /// Creates an iterator that iterates through the keys and mutable values of the ConstLru from most-recently-used to least-recently-used
    ///
    /// Does not change the LRU order of the elements.
    ///
    /// Double-ended: reversing iterates from least-recently-used to most-recently-used
    pub fn iter_mut(&mut self) -> IterMut<K, V, CAP, I> {
        IterMut::new(self)
    }

    pub fn cap(&self) -> I {
        I::from(CAP).unwrap()
    }

    pub fn len(&self) -> I {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == I::zero()
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.cap()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl<K: Eq + Clone, V: Clone, const CAP: usize, I: PrimInt + Unsigned> Clone
    for ConstLru<K, V, CAP, I>
{
    fn clone(&self) -> Self {
        let mut res = Self {
            len: self.len,
            head: self.head,
            tail: self.tail,
            nexts: self.nexts,
            prevs: self.prevs,
            keys: unsafe { MaybeUninit::uninit().assume_init() },
            values: unsafe { MaybeUninit::uninit().assume_init() },
        };
        for (i, k, v) in IterIndexed::new(self) {
            res.keys[i.to_usize().unwrap()].write(k.clone());
            res.values[i.to_usize().unwrap()].write(v.clone());
        }
        res
    }
}

impl<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> Default for ConstLru<K, V, CAP, I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> Drop for ConstLru<K, V, CAP, I> {
    fn drop(&mut self) {
        for (k, v) in IterMaybeUninit::new(self) {
            unsafe {
                k.assume_init_drop();
                v.assume_init_drop();
            }
        }
    }
}

impl<K: Eq, V, const CAP: usize, I: PrimInt + Unsigned> IntoIterator for ConstLru<K, V, CAP, I> {
    type Item = <IntoIter<K, V, CAP, I> as Iterator>::Item;

    type IntoIter = IntoIter<K, V, CAP, I>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertReplaced<K, V> {
    LruEvicted(K, V),
    OldValue(V),
}
