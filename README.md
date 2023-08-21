# const-lru

Simple `no_std` constant capacity LRU cache backed by a couple of const arrays.

Use of const generics results in allocation for max capacity being done up-front.

**This crate uses unsafe rust**

## Design

The LRU cache struct is laid out in a struct-of-arrays format: all keys are in 1 array, all values are in another array.

A sorted index over the keys is also stored in the struct to allow for `O(log N)` lookup times, where `N` is the number of elements in the LRU cache. 

LRU-ordering is implemented using a doubly-linked list, but with array indices instead of pointers. Following the struct-of-arrays format, all the next-link array indices are in one array while all the prev-link array indices are in another array.

To maximize space-efficiency and hence cache-friendliness, the last optional generic `I` specifies the index type, which can be set to an unsigned primitive int type with smaller bitwidth than `usize`, as long as it's wide enough to store the cache's capacity.

```rust
use const_lru::ConstLru;

let c: ConstLru<u8, u8, 255> = ConstLru::new(); // size = 4616, align = 8
let c: ConstLru<u8, u8, 255, u8> = ConstLru::new(); // size = 1023, align = 1
```

## Time complexity

where `N` is number of elements:
- `O(log N)` lookup using the sorted index
- `O(log N)` lookup using the sorted index
- `O(log N)` lookup using the sorted index
- `O(1)` length fetching since it's stored in the struct
