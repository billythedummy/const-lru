#![no_std]
#![doc = include_str!("../README.md")]

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::mem::MaybeUninit;
use core::ptr::{self, addr_of_mut};
use num_traits::{PrimInt, Unsigned};

mod entry;
mod errs;
mod iters;

pub use entry::*;
pub use errs::*;
pub use iters::into_iter::IntoIter;
pub use iters::iter::Iter;
pub use iters::iter_key_order::IterKeyOrder;
pub use iters::iter_key_order_mut::IterKeyOrderMut;
pub use iters::iter_mut::IterMut;

use iters::iter_key_order::IterIndexed;
use iters::iter_maybe_uninit::IterMaybeUninit;

/// Constant capacity key-addressed LRU cache.
///
/// Generics:
/// - `K`. Type of key. `Ord` is used for lookup and to address entries.
/// - `V`. Type of value.
/// - `CAP`. Capacity of the cache.
/// - `I`. Type of the index used. Must be an unsigned primitive type with bitwidth <= `usize`'s bitwidth.
#[derive(Debug)]
pub struct ConstLru<K, V, const CAP: usize, I: PrimInt + Unsigned = usize> {
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

    /// binary search index
    bs_index: [I; CAP],

    /// disregard if value == CAP
    nexts: [I; CAP],

    /// disregard if value == CAP
    prevs: [I; CAP],

    keys: [MaybeUninit<K>; CAP],

    values: [MaybeUninit<V>; CAP],
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> ConstLru<K, V, CAP, I> {
    /// Creates a new empty `ConstLru` on the stack
    ///
    /// panics if
    /// - `CAP > I::MAX`
    /// - `I::MAX > usize::MAX`
    ///
    /// WARNING: this might result in runtime stack overflow errors for large `CAP`.
    /// Use [`Self::init_at_alloc`] to initialize larger variants at preallocated memory
    pub fn new() -> Self {
        let mut res: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            Self::init_at_alloc(res.as_mut_ptr());
            res.assume_init()
        }
    }

    /// Initializes the ConstLru at a region of allocated memory
    ///
    /// # Safety
    /// `ptr` must point to uninitialized memory, since init()
    /// overwrites the data at `ptr`
    ///
    /// panics if
    /// - `CAP > I::MAX`
    /// - `I::MAX > usize::MAX`
    ///
    /// Example:
    ///
    /// ```
    /// use const_lru::ConstLru;
    /// use std::alloc::{alloc, Layout};
    ///
    /// let layout = Layout::new::<ConstLru<u32, u16, 1_000, u16>>();
    /// let container: Box<ConstLru<u32, u16, 1_000, u16>> = unsafe {
    ///     let ptr = alloc(layout) as *mut ConstLru<u32, u16, 1_000, u16>;
    ///     ConstLru::init_at_alloc(ptr);
    ///     Box::from_raw(ptr)
    /// };
    /// ```
    pub unsafe fn init_at_alloc(ptr: *mut Self) {
        // using as_mut_ptr from MaybeUninit is UB,
        // initialize fields using addr_of_mut!()

        let i_max = I::max_value()
            .to_usize()
            .unwrap_or_else(|| panic!("I::MAX > usize::MAX"));
        if CAP > i_max {
            panic!("CAP > I::MAX");
        }

        let cap = I::from(CAP).unwrap();

        addr_of_mut!((*ptr).len).write(I::zero());
        addr_of_mut!((*ptr).head).write(cap);
        addr_of_mut!((*ptr).tail).write(I::zero());

        // nexts = [1, 2, ..., cap-1, cap]
        for i in 0..CAP {
            addr_of_mut!((*ptr).nexts[i]).write(I::from(i + 1).unwrap());
        }

        // prevs = [cap, 0, 1, ..., cap-2]
        if CAP > 0 {
            addr_of_mut!((*ptr).prevs[0]).write(cap);
            for i in 1..CAP {
                addr_of_mut!((*ptr).prevs[i]).write(I::from(i - 1).unwrap());
            }
        }

        // bs_index = [cap, ..., cap]
        // UB if not initialized
        for i in 0..CAP {
            addr_of_mut!((*ptr).bs_index[i]).write(cap);
        }

        // keys and values should remain uninitialized
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

    /// Cleanup for drop impl. Drops keys and values.
    /// Other fields should be all primitive types
    fn drop_cleanup(&mut self) {
        for (k, v) in IterMaybeUninit::new(self) {
            unsafe {
                k.assume_init_drop();
                v.assume_init_drop();
            }
        }
    }

    /// Creates an iterator that iterates through the keys and values of the `ConstLru` from most-recently-used to least-recently-used
    ///
    /// Does not change the LRU order of the elements.
    ///
    /// Double-ended: reversing iterates from least-recently-used to most-recently-used
    pub fn iter(&self) -> Iter<K, V, CAP, I> {
        Iter::new(self)
    }

    /// Creates an iterator that iterates through the keys and mutable values of the `ConstLru` from most-recently-used to least-recently-used
    ///
    /// Does not change the LRU order of the elements, even if mutated.
    ///
    /// Double-ended: reversing iterates from least-recently-used to most-recently-used
    pub fn iter_mut(&mut self) -> IterMut<K, V, CAP, I> {
        IterMut::new(self)
    }

    /// Creates an iterator that iterates through the keys and values of the `ConstLru` in the order of its keys
    ///
    /// Does not change the LRU order of the elements.
    ///
    /// Double-ended: reversing iterates from descending order of its keys
    pub fn iter_key_order(&self) -> IterKeyOrder<K, V, CAP, I> {
        IterKeyOrder::new(self)
    }

    /// Creates an iterator that iterates through the keys and mutable values of the `ConstLru` in the order of its keys
    ///
    /// Does not change the LRU order of the elements, even if mutated.
    ///
    /// Double-ended: reversing iterates from descending order of its keys
    pub fn iter_key_order_mut(&mut self) -> IterKeyOrderMut<K, V, CAP, I> {
        IterKeyOrderMut::new(self)
    }

    /// Clears the `ConstLru`, removing all key-value pairs.
    pub fn clear(&mut self) {
        self.drop_cleanup();
        let ptr_to_self: *mut Self = self;
        unsafe { Self::init_at_alloc(ptr_to_self) }
    }

    /// Returns the maximum number of elements this `ConstLru` can hold
    pub fn cap(&self) -> I {
        I::from(CAP).unwrap()
    }

    /// Returns `true` if the `ConstLru` contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == I::zero()
    }

    /// Returns `true` if the `ConstLru` has reached max capacity.
    pub fn is_full(&self) -> bool {
        self.len() == self.cap()
    }

    /// Returns the number of elements in the `ConstLru`.
    pub fn len(&self) -> I {
        self.len
    }

    /// Assumes `index` is of a valid node
    /// Moves `index` to MRU position
    fn insert_replace_value(&mut self, index: I, replacement: V) -> V {
        let old_v = unsafe { self.values[index.to_usize().unwrap()].assume_init_mut() };
        let old_v_out = core::mem::replace(old_v, replacement);
        self.move_to_head(index);
        old_v_out
    }

    // Assumes N > 0 and self is not full
    // Moves newly inserted elem to MRU position
    // Returns index entry was inserted into
    fn insert_alloc_new(&mut self, insert_bs_i: I, k: K, v: V) -> I {
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

        if insert_bs_i < self.len {
            // shift everything between [bs_i, len) right
            unsafe {
                let insert_bs_i_ptr = self
                    .bs_index
                    .as_mut_ptr()
                    .add(insert_bs_i.to_usize().unwrap());
                ptr::copy(
                    insert_bs_i_ptr,
                    insert_bs_i_ptr.add(1),
                    (self.len - insert_bs_i).to_usize().unwrap(),
                );
            }
        }
        self.bs_index[insert_bs_i.to_usize().unwrap()] = free_index;

        self.len = self.len + I::one();

        self.move_to_head(self.tail);
        free_index
    }

    // Assumes index tuple is of a valid node. Should be result of Ok returned by self.get_index_of()
    fn remove_by_index(&mut self, (index, bs_i): (I, I)) -> (K, V) {
        let i = index.to_usize().unwrap();

        let key = unsafe { self.keys[i].assume_init_read() };
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

        unsafe {
            let bs_i_ptr = self.bs_index.as_mut_ptr().add(bs_i.to_usize().unwrap());
            // shift everything left to fill bs_i
            ptr::copy(
                bs_i_ptr.add(1),
                bs_i_ptr,
                (self.len - bs_i - I::one()).to_usize().unwrap(),
            );
        }

        self.len = self.len - I::one();
        (key, val)
    }

    /// Assumes index is valid
    fn get_by_index(&self, index: I) -> &V {
        unsafe { self.values[index.to_usize().unwrap()].assume_init_ref() }
    }

    /// Assumes index is valid
    fn get_mut_by_index(&mut self, index: I) -> &mut V {
        unsafe { self.values[index.to_usize().unwrap()].assume_init_mut() }
    }
}

impl<K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> ConstLru<K, V, CAP, I> {
    /// Inserts a key-value pair into the map. The entry is moved to the most-recently-used slot
    ///
    /// If `CAP == 0`, `None` is returned.
    ///
    /// If the map did not have this key present and is not full, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned in a [`InsertReplaced::OldValue`].
    /// The key is not updated, though; this matters for types that can be `==` without being identical.
    ///
    /// If the map is full, the least-recently used key-value pair is evicted and returned in a [`InsertReplaced::LruEvicted`].
    pub fn insert(&mut self, k: K, v: V) -> Option<InsertReplaced<K, V>> {
        if CAP == 0 {
            return None;
        }
        let insert_bs_i = match self.get_index_of(&k) {
            Ok((existing_index, _)) => {
                return Some(InsertReplaced::OldValue(
                    self.insert_replace_value(existing_index, v),
                ))
            }
            Err(i) => i,
        };
        if self.is_full() {
            let (_, (old_k, old_v)) = self.insert_evict_lru(insert_bs_i, k, v);
            Some(InsertReplaced::LruEvicted(old_k, old_v))
        } else {
            self.insert_alloc_new(insert_bs_i, k, v);
            None
        }
    }

    /// Assumes N > 0 and self is full
    /// Moves newly inserted elem to MRU position
    ///
    /// Returns (index entry was inserted into, evicted entry)
    fn insert_evict_lru(&mut self, insert_bs_i: I, k: K, v: V) -> (I, (K, V)) {
        // N > 0, tail must be valid
        let i = self.tail;
        let t = i.to_usize().unwrap();
        let evicted_k = unsafe { self.keys[t].assume_init_read() };
        let evicted_v = unsafe { self.values[t].assume_init_read() };
        let Ok((_should_be_t, evicted_bs_i)) = self.get_index_of(&evicted_k) else {
            unreachable!()
        };
        self.keys[t].write(k);
        self.values[t].write(v);

        match insert_bs_i.cmp(&evicted_bs_i) {
            // nothing to be done, bs_index[insert_bs_i] already == tail
            Ordering::Equal => (),
            Ordering::Less => {
                // shift everything between [insert_bs_i, evicted_bs_i) right
                // then insert at insert_bs_i
                let b = insert_bs_i.to_usize().unwrap();
                unsafe {
                    let bs_i_ptr = self.bs_index.as_mut_ptr().add(b);
                    ptr::copy(
                        bs_i_ptr,
                        bs_i_ptr.add(1),
                        (evicted_bs_i - insert_bs_i).to_usize().unwrap(),
                    );
                }
                self.bs_index[b] = self.tail;
            }
            Ordering::Greater => {
                // shift everything between (evicted_bs_i, bs_i - 1] left
                // then insert at bs_i - 1

                // safety: greater, so bs_i must be > 0
                let inser_bs_i_sub_1 = insert_bs_i - I::one();
                unsafe {
                    let evicted_bs_i_ptr = self
                        .bs_index
                        .as_mut_ptr()
                        .add(evicted_bs_i.to_usize().unwrap());
                    ptr::copy(
                        evicted_bs_i_ptr.add(1),
                        evicted_bs_i_ptr,
                        (inser_bs_i_sub_1 - evicted_bs_i).to_usize().unwrap(),
                    );
                }
                self.bs_index[inser_bs_i_sub_1.to_usize().unwrap()] = self.tail;
            }
        }

        self.move_to_head(self.tail);
        (i, (evicted_k, evicted_v))
    }

    /// Removes a key from the `ConstLru`, returning the value at the key if the key was previously in the `ConstLru`.
    pub fn remove<Q: Ord + ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let tup = self.get_index_of(k).ok()?;
        Some(self.remove_by_index(tup).1)
    }

    /// Returns a reference to the value corresponding to the key and moves entry to most-recently-used slot.
    ///
    /// To not update to most-recently-used, use [`Self::get_untouched`]
    pub fn get<Q: Ord + ?Sized>(&mut self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let (index, _) = self.get_index_of(k).ok()?;
        self.move_to_head(index);
        Some(self.get_by_index(index))
    }

    /// Returns a mutable reference to the value corresponding to the key and moves entry to most-recently-used slot.
    ///
    /// To not update to most-recently-used, use [`Self::get_mut_untouched`]
    pub fn get_mut<Q: Ord + ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let (index, _) = self.get_index_of(k).ok()?;
        self.move_to_head(index);
        Some(self.get_mut_by_index(index))
    }

    /// Ok(kv_i, bs_index_i)
    ///
    /// Err(bs_index_i)
    fn get_index_of<Q: Ord + ?Sized>(&self, k: &Q) -> Result<(I, I), I>
    where
        K: Borrow<Q>,
    {
        let l = self.len().to_usize().unwrap();
        let valid_bs_index = &self.bs_index[0..l];
        valid_bs_index
            .binary_search_by(|probe_index| {
                let p = probe_index.to_usize().unwrap();
                let probe = unsafe { self.keys[p].assume_init_ref() };
                probe.borrow().cmp(k)
            })
            .map(|bs_i| (self.bs_index[bs_i], I::from(bs_i).unwrap()))
            .map_err(|new_bsi| I::from(new_bsi).unwrap())
    }

    /// Returns a reference to the value corresponding to the key without updating the entry to most-recently-used slot
    ///
    /// To update to most-recently-used, use [`Self::get`]
    pub fn get_untouched<Q: Ord + ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let (index, _) = self.get_index_of(k).ok()?;
        Some(self.get_by_index(index))
    }

    /// Returns a mutable reference to the value corresponding to the key without updating the entry to most-recently-used slot
    ///
    /// To update to most-recently-used, use [`Self::get_mut`]
    pub fn get_mut_untouched<Q: Ord + ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let (index, _) = self.get_index_of(k).ok()?;
        Some(self.get_mut_by_index(index))
    }

    /// Gets the given key’s corresponding entry in the map for in-place manipulation.
    ///
    /// **panics** if CAP == 0
    pub fn entry(&mut self, k: K) -> Entry<'_, K, V, CAP, I> {
        Entry::new(self, k)
    }
}

impl<K: Clone, V: Clone, const CAP: usize, I: PrimInt + Unsigned> ConstLru<K, V, CAP, I> {
    /// Clones the ConstLru to a region of allocated memory
    ///
    /// # Safety
    /// `dst` must point to uninitialized memory, since this
    /// overwrites the data at `dst`
    pub unsafe fn clone_to_alloc(&self, dst: *mut Self) {
        addr_of_mut!((*dst).len).write(self.len);
        addr_of_mut!((*dst).head).write(self.head);
        addr_of_mut!((*dst).tail).write(self.tail);

        // .write(self.nexts) result in stack overflow for large CAP, so use raw memmove
        ptr::copy(
            self.nexts.as_ptr(),
            addr_of_mut!((*dst).nexts) as *mut I,
            CAP,
        );
        ptr::copy(
            self.prevs.as_ptr(),
            addr_of_mut!((*dst).prevs) as *mut I,
            CAP,
        );
        ptr::copy(
            self.bs_index.as_ptr(),
            addr_of_mut!((*dst).bs_index) as *mut I,
            CAP,
        );

        for (index, k, v) in IterIndexed::new(self) {
            let i = index.to_usize().unwrap();
            addr_of_mut!((*dst).keys[i]).write(MaybeUninit::new(k.clone()));
            addr_of_mut!((*dst).values[i]).write(MaybeUninit::new(v.clone()));
        }
    }
}

/// WARNING: this might result in runtime stack overflow errors for large `CAP`.
/// To clone a large `ConstLru`, use [`ConstLru::clone_to_alloc`]
impl<K: Clone, V: Clone, const CAP: usize, I: PrimInt + Unsigned> Clone for ConstLru<K, V, CAP, I> {
    fn clone(&self) -> Self {
        let mut res: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            self.clone_to_alloc(res.as_mut_ptr());
            res.assume_init()
        }
    }
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> Default for ConstLru<K, V, CAP, I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> Drop for ConstLru<K, V, CAP, I> {
    fn drop(&mut self) {
        self.drop_cleanup();
    }
}

impl<K, V, const CAP: usize, I: PrimInt + Unsigned> IntoIterator for ConstLru<K, V, CAP, I> {
    type Item = <IntoIter<K, V, CAP, I> as Iterator>::Item;

    type IntoIter = IntoIter<K, V, CAP, I>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

/// Optional return type of [`ConstLru::insert`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertReplaced<K, V> {
    LruEvicted(K, V),
    OldValue(V),
}

/// Creates a full ConstLru cache from an `entries` array.
///
/// Assumes `entries` is in MRU -> LRU order.
///
/// Returns error if duplicate keys found.
///
/// WARNING: this might result in runtime stack overflow errors for large `CAP`.
impl<K: Ord, V, const CAP: usize, I: PrimInt + Unsigned> TryFrom<[(K, V); CAP]>
    for ConstLru<K, V, CAP, I>
{
    type Error = DuplicateKeysError<K>;

    fn try_from(entries: [(K, V); CAP]) -> Result<Self, Self::Error> {
        // entries need to fit on the stack too, so Self::new() shouldn't stack overflow
        let mut res = Self::new();
        res.len = res.cap();
        res.head = I::zero();
        res.tail = if CAP > 0 {
            res.len - I::one()
        } else {
            I::zero()
        };

        for (i, (k, v)) in entries.into_iter().enumerate() {
            res.keys[i].write(k);
            res.values[i].write(v);
        }

        for (i, val) in res.bs_index.iter_mut().enumerate() {
            *val = I::from(i).unwrap();
        }
        res.bs_index.sort_unstable_by(|a, b| {
            let k_a = unsafe { res.keys[a.to_usize().unwrap()].assume_init_ref() };
            let k_b = unsafe { res.keys[b.to_usize().unwrap()].assume_init_ref() };
            k_a.cmp(k_b)
        });

        if CAP > 1 {
            for w in res.bs_index.windows(2) {
                let index_1 = w[0];
                let i1 = index_1.to_usize().unwrap();
                let i2 = w[1].to_usize().unwrap();
                let k1 = unsafe { res.keys[i1].assume_init_ref() };
                let k2 = unsafe { res.keys[i2].assume_init_ref() };
                if k1 == k2 {
                    // remove from list so no double free
                    res.unlink_node(index_1);
                    res.len = res.len - I::one();

                    // cleanup value
                    unsafe { res.values[i1].assume_init_drop() };
                    let k_copied_out = unsafe { res.keys[i1].assume_init_read() };
                    return Err(DuplicateKeysError(k_copied_out));
                }
            }
        }

        Ok(res)
    }
}
