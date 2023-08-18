# const-lru

Simple `no_std` constant capacity LRU cache.

Use of const generics results in allocation for max capacity being done up-front.

**This crate uses unsafe rust**

## Time complexity

where `N` is number of elements:
- `O(N)` lookup
- `O(N)` insertion
- `O(N)` deletion
