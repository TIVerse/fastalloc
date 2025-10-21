//! Builder for pool configuration.

use super::{GrowthStrategy, InitializationStrategy, PoolConfig};
use crate::error::{Error, Result};
use crate::utils::validate_alignment;
use core::mem;

/// Builder for constructing a `PoolConfig` with validation.
///
/// # Examples
///
/// ```rust
/// use fastalloc::{PoolConfig, GrowthStrategy};
///
/// let config = PoolConfig::<i32>::builder()
///     .capacity(1000)
///     .max_capacity(Some(10000))
///     .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
///     .alignment(64)
///     .pre_initialize(true)
///     .build()
///     .unwrap();
/// ```
pub struct PoolConfigBuilder<T> {
    capacity: Option<usize>,
    max_capacity: Option<usize>,
    growth_strategy: GrowthStrategy,
    alignment: usize,
    pre_initialize: bool,
    initialization_strategy: InitializationStrategy<T>,
    thread_local: bool,
}

impl<T> PoolConfigBuilder<T> {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self {
            capacity: None,
            max_capacity: None,
            growth_strategy: GrowthStrategy::None,
            alignment: mem::align_of::<T>(),
            pre_initialize: false,
            initialization_strategy: InitializationStrategy::Lazy,
            thread_local: false,
        }
    }

    /// Sets the initial capacity of the pool.
    ///
    /// This is a required setting and must be at least 1.
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Sets the maximum capacity of the pool.
    ///
    /// If set, the pool will not grow beyond this size.
    /// If `None`, the pool can grow indefinitely (subject to memory availability).
    pub fn max_capacity(mut self, max_capacity: Option<usize>) -> Self {
        self.max_capacity = max_capacity;
        self
    }

    /// Sets the growth strategy for the pool.
    pub fn growth_strategy(mut self, strategy: GrowthStrategy) -> Self {
        self.growth_strategy = strategy;
        self
    }

    /// Sets the memory alignment for pool objects.
    ///
    /// Must be a power of two. Defaults to the natural alignment of `T`.
    pub fn alignment(mut self, alignment: usize) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets whether objects should be pre-initialized when the pool is created.
    ///
    /// If `true`, all initial capacity will be allocated and initialized eagerly.
    /// If `false`, objects are initialized on first use.
    pub fn pre_initialize(mut self, pre_initialize: bool) -> Self {
        self.pre_initialize = pre_initialize;
        self
    }

    /// Sets a custom initializer function for creating objects.
    pub fn initializer(mut self, initializer: impl Fn() -> T + Send + Sync + 'static) -> Self {
        self.initialization_strategy = InitializationStrategy::eager(initializer);
        self
    }

    /// Sets a custom reset function to be called when objects are returned to the pool.
    pub fn reset_fn(
        mut self,
        initializer: impl Fn() -> T + Send + Sync + 'static,
        reset: impl Fn(&mut T) + Send + Sync + 'static,
    ) -> Self {
        self.initialization_strategy = InitializationStrategy::custom(initializer, reset);
        self
    }

    /// Sets the initialization strategy directly.
    pub fn initialization_strategy(mut self, strategy: InitializationStrategy<T>) -> Self {
        self.initialization_strategy = strategy;
        self
    }

    /// Sets whether this should be a thread-local pool.
    ///
    /// Thread-local pools avoid synchronization overhead but can only be used
    /// from the thread that created them.
    #[cfg(feature = "std")]
    pub fn thread_local(mut self, thread_local: bool) -> Self {
        self.thread_local = thread_local;
        self
    }

    /// Builds the configuration, validating all parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Capacity is not set or is 0
    /// - Alignment is not a power of two
    /// - Max capacity is less than initial capacity
    pub fn build(self) -> Result<PoolConfig<T>> {
        // Validate capacity
        let capacity = self
            .capacity
            .ok_or_else(|| Error::invalid_config("capacity must be specified"))?;

        if capacity == 0 {
            return Err(Error::invalid_config("capacity must be at least 1"));
        }

        // Validate max_capacity
        if let Some(max) = self.max_capacity {
            if max < capacity {
                return Err(Error::invalid_config("max_capacity must be >= capacity"));
            }
        }

        // Validate alignment
        validate_alignment(self.alignment)?;

        // Ensure pre_initialize and initialization strategy are consistent
        let initialization_strategy =
            if self.pre_initialize && self.initialization_strategy.is_lazy() {
                // If pre_initialize is true but strategy is lazy, upgrade to eager with default
                InitializationStrategy::Lazy // Will be handled by pool implementation
            } else {
                self.initialization_strategy
            };

        Ok(PoolConfig {
            capacity,
            max_capacity: self.max_capacity,
            growth_strategy: self.growth_strategy,
            alignment: self.alignment,
            pre_initialize: self.pre_initialize,
            initialization_strategy,
            thread_local: self.thread_local,
        })
    }
}

impl<T> Default for PoolConfigBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_requires_capacity() {
        let result = PoolConfig::<i32>::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_rejects_zero_capacity() {
        let result = PoolConfig::<i32>::builder().capacity(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_validates_alignment() {
        let result = PoolConfig::<i32>::builder()
            .capacity(100)
            .alignment(7) // Not a power of two
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_validates_max_capacity() {
        let result = PoolConfig::<i32>::builder()
            .capacity(100)
            .max_capacity(Some(50)) // Less than capacity
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_creates_valid_config() {
        let config = PoolConfig::<i32>::builder()
            .capacity(100)
            .max_capacity(Some(1000))
            .alignment(64)
            .pre_initialize(true)
            .build()
            .unwrap();

        assert_eq!(config.capacity(), 100);
        assert_eq!(config.max_capacity(), Some(1000));
        assert_eq!(config.alignment(), 64);
        assert!(config.pre_initialize());
    }

    #[test]
    fn builder_with_growth_strategy() {
        let config = PoolConfig::<i32>::builder()
            .capacity(100)
            .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
            .build()
            .unwrap();

        assert!(config.growth_strategy().allows_growth());
    }
}
