# Change Log

## [0.2.6] - 2026-07-23

### Changed

- License is now MIT or Apache.

## [0.2.5] - 2026-07-13

### Improved

* `get` and `peek` now recover gracefully from mutex poisoning, mirroring the
  behavior of the `Drop` implementation of `SharedReducer`; they no longer
  panic.

### Fixed

* Typos in the API documentation and in `README.md`.

## [0.2.4] - 2026-02-15

### New

* A few `Debug` impls.

### Changed

* 2024 edition, Rust 1.85.

### Improved

* `Reduce` now can take a closure.

* Mutex poisoning is now handled gracefully in `Drop` impl.

## [0.2.3] - 2025-04-15

### New

* Global and local values have different type parameters.

### Changed

* Renamed crate from `buf-mutex` to `openmp-reducer`.

## [0.2.2] - 2025-04-11

### Changed

* Added references to OpenMP and Cilk.

## [0.2.1] - 2025-02-05

### Fixed

* Fixed API docs.

## [0.2.0] - 2025-02-05

### Improved

* Better APIs and compile-time check for shared copies.

## [0.1.0] - 2025-02-04

### New

* First release.
