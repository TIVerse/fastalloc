//! Initialization strategies for pool objects.

use alloc::boxed::Box;

/// Strategy for initializing objects in a memory pool.
///
/// # Examples
///
/// ```rust
/// use fastalloc::InitializationStrategy;
///
/// // Lazy initialization (default)
/// let strategy = InitializationStrategy::<i32>::Lazy;
///
/// // Eager initialization with a default value
/// let strategy = InitializationStrategy::Eager {
///     initializer: Box::new(|| 42),
/// };
///
/// // Custom initialization with reset callback
/// let strategy = InitializationStrategy::Custom {
///     initializer: Box::new(|| vec![1, 2, 3]),
///     reset: Some(Box::new(|v| v.clear())),
/// };
/// ```
pub enum InitializationStrategy<T> {
    /// Initialize objects only when first allocated (lazy).
    Lazy,

    /// Initialize all objects eagerly when pool is created.
    Eager {
        /// Function to create initial values
        initializer: Box<dyn Fn() -> T + Send + Sync>,
    },

    /// Custom initialization with optional reset function.
    Custom {
        /// Function to create initial values
        initializer: Box<dyn Fn() -> T + Send + Sync>,
        /// Optional function to reset objects when returned to pool
        #[allow(clippy::type_complexity)]
        reset: Option<Box<dyn Fn(&mut T) + Send + Sync>>,
    },
}

impl<T> InitializationStrategy<T> {
    /// Creates an eager initialization strategy with the given initializer.
    pub fn eager(initializer: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self::Eager {
            initializer: Box::new(initializer),
        }
    }

    /// Creates a custom initialization strategy with both initializer and reset function.
    pub fn custom(
        initializer: impl Fn() -> T + Send + Sync + 'static,
        reset: impl Fn(&mut T) + Send + Sync + 'static,
    ) -> Self {
        Self::Custom {
            initializer: Box::new(initializer),
            reset: Some(Box::new(reset)),
        }
    }

    /// Creates a custom initialization strategy with only an initializer.
    pub fn custom_init_only(initializer: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self::Custom {
            initializer: Box::new(initializer),
            reset: None,
        }
    }

    /// Returns whether this strategy is lazy.
    #[inline]
    pub fn is_lazy(&self) -> bool {
        matches!(self, InitializationStrategy::Lazy)
    }

    /// Returns whether this strategy is eager.
    #[inline]
    pub fn is_eager(&self) -> bool {
        matches!(self, InitializationStrategy::Eager { .. })
    }

    /// Creates an initial value if an initializer is available.
    pub fn initialize(&self) -> Option<T> {
        match self {
            InitializationStrategy::Lazy => None,
            InitializationStrategy::Eager { initializer } => Some(initializer()),
            InitializationStrategy::Custom { initializer, .. } => Some(initializer()),
        }
    }

    /// Resets an object using the reset function, if available.
    pub fn reset(&self, value: &mut T) {
        if let InitializationStrategy::Custom {
            reset: Some(reset_fn),
            ..
        } = self
        {
            reset_fn(value);
        }
    }
}

impl<T> Default for InitializationStrategy<T> {
    fn default() -> Self {
        Self::Lazy
    }
}

impl<T> core::fmt::Debug for InitializationStrategy<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InitializationStrategy::Lazy => write!(f, "InitializationStrategy::Lazy"),
            InitializationStrategy::Eager { .. } => {
                write!(f, "InitializationStrategy::Eager {{ .. }}")
            }
            InitializationStrategy::Custom { reset, .. } => f
                .debug_struct("InitializationStrategy::Custom")
                .field("has_reset", &reset.is_some())
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn lazy_strategy() {
        let strategy = InitializationStrategy::<i32>::Lazy;
        assert!(strategy.is_lazy());
        assert!(!strategy.is_eager());
        assert!(strategy.initialize().is_none());
    }

    #[test]
    fn eager_strategy() {
        let strategy = InitializationStrategy::eager(|| 42);
        assert!(!strategy.is_lazy());
        assert!(strategy.is_eager());
        assert_eq!(strategy.initialize(), Some(42));
    }

    #[test]
    fn custom_strategy_with_reset() {
        let strategy = InitializationStrategy::custom(|| vec![1, 2, 3], |v| v.clear());

        let mut value = strategy.initialize().unwrap();
        assert_eq!(value, vec![1, 2, 3]);

        value.push(4);
        strategy.reset(&mut value);
        assert_eq!(value, Vec::<i32>::new());
    }

    #[test]
    fn custom_strategy_without_reset() {
        let strategy = InitializationStrategy::custom_init_only(|| 100);

        let value = strategy.initialize().unwrap();
        assert_eq!(value, 100);

        let mut value = 200;
        strategy.reset(&mut value);
        assert_eq!(value, 200); // No reset function, so unchanged
    }
}
