# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - 2025-10-21

### Changed

- Migrated repository from GitLab to GitHub at https://github.com/TIVerse/fastalloc
- Updated all documentation and links to point to the new repository location

## [1.0.0] - 2024-10-16

### Changed

- **STABLE RELEASE**: First stable version with stable API
- Improved `Poolable` trait design - removed blanket implementation to prevent conflicts
- Added `Poolable` implementations for common types (primitives, String, Vec, Box, Option)
- Enhanced async compatibility with better lifetime management
- Fixed RefCell borrowing issues in `GrowingPool` 
- Improved thread safety and lifetime handling across all pool types

### Fixed

- Fixed compilation errors with thread-safe pools in async contexts
- Resolved lifetime issues in handle management
- Fixed borrowing conflicts in growing pool allocation
- Corrected example code for proper `Poolable` trait usage
- Fixed async example to properly drop handles before await points

### Added

- Explicit `Poolable` trait implementations for user types in examples
- Better documentation for async usage patterns
- Improved error messages and debug information

### Performance

- Optimized allocation path to avoid unnecessary borrows
- Reduced lock contention in `GrowingPool`
- Better cache locality with improved memory layout

## [0.1.0] - 2024-10-16

### Added

- Initial release of fastalloc
- `FixedPool` with O(1) allocation and deallocation
- `GrowingPool` with configurable growth strategies
- `ThreadLocalPool` for zero-synchronization performance
- `ThreadSafePool` for concurrent access
- Multiple allocation strategies: Stack (LIFO), free-list, and bitmap
- Smart handles with RAII automatic return to pool
- Builder pattern for pool configuration
- Growth strategies: None, Linear, Exponential, and Custom
- Initialization strategies: Lazy, Eager, and Custom with reset callbacks
- Optional statistics collection (`stats` feature)
- Optional serialization support (`serde` feature)
- Optional faster locks (`parking_lot` feature)
- Optional lock-free pool (`lock-free` feature with `crossbeam`)
- Optional tracing instrumentation (`tracing` feature)
- `no_std` support (core + alloc)
- Comprehensive documentation with examples
- Extensive test suite including integration tests
- Criterion-based benchmarks
- Examples for common use cases

### Features

- Zero-cost abstractions with type-safe handles
- Configurable memory alignment (including cache-line alignment)
- Custom initializers and reset functions
- Pool exhaustion detection with helpful error messages
- Debug assertions for double-free and use-after-free detection
- Thread-safe and thread-local variants

### Performance

- Fixed pool allocation: < 20ns per object
- Deallocation: < 10ns per object
- Memory overhead: < 5% for pools over 1000 objects
- Thread-safe pool: < 100ns with moderate contention

### Documentation

- Comprehensive API documentation
- Getting started guide
- Performance tuning guide
- Migration guide from standard allocation
- Architecture documentation
- Benchmark methodology
- Contributing guidelines

[Unreleased]: https://github.com/TIVerse/fastalloc/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/TIVerse/fastalloc/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/TIVerse/fastalloc/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/TIVerse/fastalloc/releases/tag/v0.1.0
