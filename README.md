# const-lru

A simple no_std, non-hashing, constant-capacity, constant-memory-usage LRU cache.

The data structure is backed by a couple of const-generic arrays, resulting in all required memory being allocated up-front.

**This crate uses unsafe rust**, but all public APIs are safe.

## Design

The LRU cache struct is laid out in a struct-of-arrays format: all keys are in 1 array, all values are in another array.

A sorted index over the keys is also stored in the struct to allow for `O(log N)` lookup times using binary search. 

LRU-ordering is implemented using a doubly-linked list, but with array indices instead of pointers. Following the struct-of-arrays format, all the next-link array indices are in one array while all the prev-link array indices are in another array.

To maximize space-efficiency, the last optional generic `I` specifies the index type, which can be set to an unsigned primitive int type with smaller bitwidth than `usize`, as long as it's wide enough to store the cache's capacity.

```rust
use const_lru::ConstLru;
use core::mem;

assert_eq!(mem::align_of::<ConstLru<u8, u8, 255>>(), 8);
assert_eq!(mem::size_of::<ConstLru<u8, u8, 255>>(), 6656);

assert_eq!(mem::align_of::<ConstLru<u8, u8, 255, u8>>(), 1);
assert_eq!(mem::size_of::<ConstLru<u8, u8, 255, u8>>(), 1278);
```

## Time complexity

where `N` is number of elements:
- Retrieval: `O(log N)` lookup using the sorted index
- Insertion: `O(log N)` lookup using the sorted index + `O(N)` to modify the sorted index (bitwise-copy of index types similar to `Vec`)
- Deletion: `O(log N)` lookup using the sorted index + `O(N)` to modify the sorted index (bitwise-copy of index types similar to `Vec`)
- Length fetching: `O(1)` since it's stored in the struct
- Retrieving MRU element: `O(1)` using `.iter().next()`
- Retrieving LRU element: `O(1)` using `.iter().next_back()`

## Motivation

Most, if not all, general LRU cache implementations (including but not limited to [associative-cache](https://docs.rs/associative-cache), [caches](https://docs.rs/caches), [clru](https://docs.rs/clru), [hashlink](https://docs.rs/hashlink), [lru](https://docs.rs/lru)) rely on one-or-more hashmaps to give `O(1)` lookup times. While fast, this makes their usage less well-suited for memory-constrained environments like embedded systems since hashmaps may rehash and reallocate more memory. `ConstLru` on the other hand is designed to have a fixed size known at compile-time, but gives up a `O(1)` hashing-based lookup for a `O(log N)` binary-search-based lookup and more expensive inserts and removes.

[uluru](https://docs.rs/uluru) is another fixed-capacity LRU-cache implementation that has `O(n)` lookup times but uses less memory.
