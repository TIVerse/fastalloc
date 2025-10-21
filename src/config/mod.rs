//! Configuration types for memory pools.

mod builder;
mod growth_strategy;
mod initialization;

pub use builder::PoolConfigBuilder;
pub use growth_strategy::GrowthStrategy;
pub use initialization::InitializationStrategy;

use core::mem;

/// Configuration for a memory pool.
///
/// Use `PoolConfig::builder()` to construct a configuration with validation.
///
/// # Examples
///
/// ```rust
/// use fastalloc::{PoolConfig, GrowthStrategy};
///
/// let config: PoolConfig<i32> = PoolConfig::builder()
///     .capacity(1000)
///     .max_capacity(Some(10000))
///     .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
///     .alignment(64)
///     .pre_initialize(true)
///     .build()
///     .unwrap();
/// ```
pub struct PoolConfig<T> {
    /// Initial capacity of the pool
    pub(crate) capacity: usize,

    /// Maximum capacity (None for unlimited)
    pub(crate) max_capacity: Option<usize>,

    /// Strategy for growing the pool
    pub(crate) growth_strategy: GrowthStrategy,

    /// Memory alignment (must be power of 2)
    pub(crate) alignment: usize,

    /// Whether to pre-initialize all objects
    pub(crate) pre_initialize: bool,

    /// Initialization strategy
    #[allow(dead_code)]
    pub(crate) initialization_strategy: InitializationStrategy<T>,

    /// Whether this is a thread-local pool
    pub(crate) thread_local: bool,
}

impl<T> PoolConfig<T> {
    /// Creates a new builder for constructing a pool configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::PoolConfig;
    ///
    /// let config = PoolConfig::<i32>::builder()
    ///     .capacity(100)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> PoolConfigBuilder<T> {
        PoolConfigBuilder::new()
    }

    /// Returns the initial capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the maximum capacity, if set.
    #[inline]
    pub fn max_capacity(&self) -> Option<usize> {
        self.max_capacity
    }

    /// Returns the growth strategy.
    #[inline]
    pub fn growth_strategy(&self) -> &GrowthStrategy {
        &self.growth_strategy
    }

    /// Returns the alignment requirement.
    #[inline]
    pub fn alignment(&self) -> usize {
        self.alignment
    }

    /// Returns whether objects should be pre-initialized.
    #[inline]
    pub fn pre_initialize(&self) -> bool {
        self.pre_initialize
    }

    /// Returns whether this is a thread-local pool configuration.
    #[inline]
    pub fn thread_local(&self) -> bool {
        self.thread_local
    }
}

impl<T> Default for PoolConfig<T> {
    fn default() -> Self {
        Self {
            capacity: 100,
            max_capacity: None,
            growth_strategy: GrowthStrategy::None,
            alignment: mem::align_of::<T>(),
            pre_initialize: false,
            initialization_strategy: InitializationStrategy::Lazy,
            thread_local: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = PoolConfig::<i32>::default();
        assert_eq!(config.capacity(), 100);
        assert_eq!(config.max_capacity(), None);
        assert_eq!(config.alignment(), mem::align_of::<i32>());
        assert!(!config.pre_initialize());
        assert!(!config.thread_local());
    }

    #[test]
    fn builder_creates_config() {
        let config = PoolConfig::<i32>::builder().capacity(500).build().unwrap();

        assert_eq!(config.capacity(), 500);
    }
}
