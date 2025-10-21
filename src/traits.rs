//! Traits for working with memory pools.

use crate::error::Result;

/// A trait for types that can be used with memory pools.
///
/// This trait provides hooks for custom initialization and cleanup logic
/// when objects are allocated from or returned to a pool.
///
/// # Examples
///
/// ```rust
/// use fastalloc::Poolable;
///
/// struct GameEntity {
///     position: (f32, f32),
///     velocity: (f32, f32),
///     health: i32,
/// }
///
/// impl Poolable for GameEntity {
///     fn on_acquire(&mut self) {
///         // Reset state when acquired from pool
///         self.position = (0.0, 0.0);
///         self.velocity = (0.0, 0.0);
///         self.health = 100;
///     }
///     
///     fn on_release(&mut self) {
///         // Cleanup before returning to pool
///         // (optional - could clear resources, etc.)
///     }
/// }
/// ```
pub trait Poolable {
    /// Called when an object is acquired from the pool.
    ///
    /// This is a good place to reset the object to a clean state.
    /// The default implementation does nothing.
    fn on_acquire(&mut self) {}

    /// Called when an object is being returned to the pool.
    ///
    /// This is a good place to perform cleanup or release resources.
    /// The default implementation does nothing.
    fn on_release(&mut self) {}
}

// Note: We don't provide a blanket implementation to allow users to implement Poolable
// for their types without conflicts. The trait has default methods so no implementation
// is required unless custom behavior is needed.

/// Internal trait for pool implementations.
///
/// This trait is not intended for direct use by library users.
#[doc(hidden)]
pub trait Pool<T> {
    /// Allocate an object from the pool with the given initial value.
    fn allocate(&self, value: T) -> Result<Self::Handle>
    where
        Self::Handle: Sized;

    /// The handle type returned by this pool.
    type Handle;
}

/// Trait for pools that support statistics collection.
#[cfg(feature = "stats")]
pub trait PoolStats {
    /// Get current statistics for this pool.
    fn statistics(&self) -> crate::stats::PoolStatistics;

    /// Reset statistics counters.
    fn reset_statistics(&self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poolable_default_impl() {
        struct TestType {
            value: i32,
        }

        impl Poolable for TestType {}

        let mut obj = TestType { value: 42 };

        // Should compile and do nothing (uses default impl)
        obj.on_acquire();
        obj.on_release();

        assert_eq!(obj.value, 42);
    }

    #[test]
    fn poolable_custom_impl() {
        struct CustomType {
            counter: i32,
        }

        impl Poolable for CustomType {
            fn on_acquire(&mut self) {
                self.counter = 0;
            }

            fn on_release(&mut self) {
                self.counter = -1;
            }
        }

        let mut obj = CustomType { counter: 100 };
        obj.on_acquire();
        assert_eq!(obj.counter, 0);

        obj.counter = 50;
        obj.on_release();
        assert_eq!(obj.counter, -1);
    }
}
