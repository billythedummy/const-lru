# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [UNRELEASED]

### Added

- `impl Display` for `DuplicateKeysError`
- `?Sized` bound for `Q` key asref generics

## [0.2.2] - 2023-08-29

### Added

- `clone_to_alloc()` method to allow cloning of large `ConstLru`s without stack overflows

### Fixed

- `clear()` causing stack overflows for large `ConstLru`s

## [0.2.1] - 2023-08-29

### Fixed

- UB involving uninitialized `bs_index` in `new()` and converting references to pointers caught by miri

## [0.2.0] - 2023-08-29

### Added

- `init_at_alloc()` fn to allow large `ConstLru`s to be initialized at pre-allocated memory without causing stack overflows.

### Changed

- `IterIndexed` now iterates in key-order instead of lru-order. This should result in slightly faster `.clone()`s

## [0.1.0] - 2023-08-23

Initial release
