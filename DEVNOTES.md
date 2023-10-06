# Development Notes

## Alternate Red-Black Tree Design

The `feat/bst` branch contains a reimplementation using a red-black tree instead of a simple binary-search index. The differences in the microbenchmarks were approx:

- red-black tree had 2x slower lookup than binary-search index for 10k items
- red-black tree had 2x slower insertions than binary-search index for 10k items
- red-black tree had 2x faster deletions than binary-search index for 10k items

Hence decided to continue using the binary-search index.
