//! # fastalloc
//!
//! A memory pooling library for Rust with type-safe handles and RAII-based memory management.
//!
//! **Version 1.5.0** - Production-ready release with performance optimizations and comprehensive documentation.
//!
//! ## Overview
//!
//! `fastalloc` provides memory pools that allow you to reuse allocations efficiently,
//! offering **1.3-1.4x faster allocation** than standard heap with the key benefits of:
//! - **Predictable latency**: No allocation spikes or fragmentation slowdowns
//! - **Cache locality**: Objects stored contiguously improve cache hit rates  
//! - **Zero fragmentation**: Eliminates long-term heap fragmentation
//! - **Real-time guarantees**: Bounded worst-case allocation time
//!
//! Designed for use cases where objects are frequently created and destroyed:
//! - Game development (entities, particles, physics objects)
//! - Real-time systems (audio processing, robotics)
//! - High-performance servers (connection pooling, request handling)
//! - Embedded systems (constrained memory, no fragmentation)
//! - Scientific computing (matrices, particles, graph nodes)
//!
//! ## Features
//!
//! - **Multiple pool types**: Fixed-size, growing, thread-local, and thread-safe pools
//! - **Type-safe handles**: RAII-based handles that automatically return objects to the pool
//! - **Flexible configuration**: Builder pattern with extensive customization options
//! - **Multiple allocation strategies**: Stack (LIFO), free-list, and bitmap allocators
//! - **Optional statistics**: Track allocation patterns and pool usage (with `stats` feature)
//! - **no_std support**: Works in embedded and bare-metal environments
//!
//! ## Quick Start
//!
//! ```rust
//! use fastalloc::{FixedPool, PoolConfig};
//!
//! // Create a pool of 1000 integers
//! let pool = FixedPool::<i32>::new(1000).unwrap();
//!
//! // Allocate from the pool
//! let mut handle = pool.allocate(42).unwrap();
//!
//! // Use the value
//! assert_eq!(*handle, 42);
//! *handle = 100;
//! assert_eq!(*handle, 100);
//!
//! // Automatically returned to pool when handle is dropped
//! drop(handle);
//! ```
//!
//! ## Builder Configuration
//!
//! ```rust
//! use fastalloc::{PoolConfig, GrowthStrategy};
//!
//! let config: PoolConfig<i32> = PoolConfig::builder()
//!     .capacity(1000)
//!     .max_capacity(Some(10000))
//!     .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
//!     .alignment(64) // Cache-line aligned
//!     .pre_initialize(true)
//!     .build()
//!     .unwrap();
//! ```
//!
//! ## Performance
//!
//! Benchmark results (criterion.rs, release mode with LTO):
//! - Fixed pool allocation: ~3.5ns per object (1.3-1.4x faster than Box)
//! - Growing pool allocation: ~4.6ns per object
//! - Allocation reuse (LIFO): ~7.2ns per cycle
//!
//! See [BENCHMARKS.md](https://github.com/TIVerse/fastalloc/blob/master/BENCHMARKS.md)
//! for detailed methodology and results.
//!
//! ## Safety
//!
//! This crate minimizes the use of `unsafe` code and leverages Rust's ownership system
//! to prevent common memory safety issues:
//!
//! - ✅ **Use-after-free**: Handles maintain exclusive ownership via borrow checker
//! - ✅ **Double-free**: Allocator tracks which slots are in use (bitmap in debug mode)
//! - ✅ **Memory leaks**: RAII ensures objects are returned to pool when dropped
//! - ✅ **Data races**: Thread-safe types use proper synchronization (Arc + Mutex)
//!
//! Debug builds include additional runtime checks:
//! - Double-free detection (O(1) bitmap check)
//! - Index bounds validation
//! - Allocation state consistency
//!
//! See [SAFETY.md](https://github.com/TIVerse/fastalloc/blob/master/SAFETY.md)
//! for detailed safety guarantees and `unsafe` code documentation.
//!
//! ## Documentation
//!
//! - [API Documentation](https://docs.rs/fastalloc) - Complete API reference
//! - [BENCHMARKS.md](https://github.com/TIVerse/fastalloc/blob/master/BENCHMARKS.md) - Benchmark results and methodology
//! - [SAFETY.md](https://github.com/TIVerse/fastalloc/blob/master/SAFETY.md) - Safety guarantees
//! - [ARCHITECTURE.md](https://github.com/TIVerse/fastalloc/blob/master/ARCHITECTURE.md) - Internal design
//! - [Examples](https://github.com/TIVerse/fastalloc/tree/master/examples) - Working code examples

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, rust_2018_idioms)]
#![allow(clippy::module_inception)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Core modules
pub mod config;
pub mod error;
pub mod handle;
pub mod pool;
pub mod traits;

// Internal modules
mod allocator;
mod utils;

// Optional modules
#[cfg(feature = "stats")]
#[cfg_attr(docsrs, doc(cfg(feature = "stats")))]
pub mod stats;

// Re-exports for convenience
pub use config::{GrowthStrategy, InitializationStrategy, PoolConfig};
pub use error::{Error, Result};
pub use handle::{OwnedHandle, SharedHandle, WeakHandle};
pub use pool::{FixedPool, GrowingPool};
pub use traits::Poolable;

#[cfg(feature = "std")]
pub use pool::{ThreadLocalPool, ThreadSafePool};

#[cfg(all(feature = "std", feature = "lock-free"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "lock-free"))))]
pub use pool::LockFreePool;

#[cfg(feature = "stats")]
pub use stats::{PoolStatistics, StatisticsCollector};

// Prelude for convenient imports
pub mod prelude {
    //! Convenient re-exports of commonly used types

    pub use crate::config::{GrowthStrategy, InitializationStrategy, PoolConfig};
    pub use crate::error::{Error, Result};
    pub use crate::handle::{OwnedHandle, SharedHandle, WeakHandle};
    pub use crate::pool::{FixedPool, GrowingPool};
    pub use crate::traits::Poolable;

    #[cfg(feature = "std")]
    pub use crate::pool::{ThreadLocalPool, ThreadSafePool};

    #[cfg(all(feature = "std", feature = "lock-free"))]
    pub use crate::pool::LockFreePool;

    #[cfg(feature = "stats")]
    pub use crate::stats::{PoolStatistics, StatisticsCollector};
}

// Provide Poolable implementations for common types

// Primitive integers
impl Poolable for i8 {}
impl Poolable for i16 {}
impl Poolable for i32 {}
impl Poolable for i64 {}
impl Poolable for i128 {}
impl Poolable for isize {}

impl Poolable for u8 {}
impl Poolable for u16 {}
impl Poolable for u32 {}
impl Poolable for u64 {}
impl Poolable for u128 {}
impl Poolable for usize {}

// Floating point
impl Poolable for f32 {}
impl Poolable for f64 {}

// Other primitives
impl Poolable for bool {}
impl Poolable for char {}

// Common standard types
#[cfg(feature = "std")]
impl Poolable for String {}
#[cfg(not(feature = "std"))]
impl Poolable for alloc::string::String {}
impl<T: Poolable> Poolable for alloc::vec::Vec<T> {}
impl<T: Poolable> Poolable for alloc::boxed::Box<T> {}
impl<T: Poolable> Poolable for Option<T> {}
impl<T: Poolable, E> Poolable for core::result::Result<T, E> {}

// Fixed-size arrays (common sizes)
impl<T: Poolable> Poolable for [T; 1] {}
impl<T: Poolable> Poolable for [T; 2] {}
impl<T: Poolable> Poolable for [T; 3] {}
impl<T: Poolable> Poolable for [T; 4] {}
impl<T: Poolable> Poolable for [T; 8] {}
impl<T: Poolable> Poolable for [T; 16] {}
impl<T: Poolable> Poolable for [T; 32] {}
impl<T: Poolable> Poolable for [T; 64] {}
impl<T: Poolable> Poolable for [T; 128] {}
impl<T: Poolable> Poolable for [T; 256] {}

// Tuples (up to 4 elements for common cases)
impl<T: Poolable, U: Poolable> Poolable for (T, U) {}
impl<T: Poolable, U: Poolable, V: Poolable> Poolable for (T, U, V) {}
impl<T: Poolable, U: Poolable, V: Poolable, W: Poolable> Poolable for (T, U, V, W) {}
