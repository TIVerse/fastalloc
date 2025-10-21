//! Thread-local memory pool implementation.

use crate::config::PoolConfig;
use crate::error::Result;
use crate::handle::OwnedHandle;
use crate::pool::FixedPool;
use std::marker::PhantomData;

/// A thread-local memory pool that avoids synchronization overhead.
///
/// Each thread gets its own pool instance, eliminating the need for locks
/// and providing the best possible allocation performance. However, objects
/// allocated from one thread's pool cannot be used in another thread.
///
/// # Examples
///
/// ```rust
/// use fastalloc::ThreadLocalPool;
///
/// // Each thread will have its own pool
/// let pool = ThreadLocalPool::<i32>::new(100).unwrap();
///
/// let mut handle = pool.allocate(42).unwrap();
/// assert_eq!(*handle, 42);
/// *handle = 100;
/// ```
///
/// # Performance
///
/// - Allocation: < 20ns per object (similar to FixedPool)
/// - Zero synchronization overhead
/// - Best choice for single-threaded or per-thread usage
///
/// # Thread Safety
///
/// `ThreadLocalPool` is `Send` but not `Sync`. You can send it to another
/// thread, but you cannot share it between threads.
pub struct ThreadLocalPool<T> {
    pool: FixedPool<T>,
    _marker: PhantomData<*const ()>, // Makes it !Sync
}

impl<T: crate::traits::Poolable> ThreadLocalPool<T> {
    /// Creates a new thread-local pool with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::ThreadLocalPool;
    ///
    /// let pool = ThreadLocalPool::<String>::new(50).unwrap();
    /// ```
    pub fn new(capacity: usize) -> Result<Self> {
        let config = PoolConfig::builder().capacity(capacity).build()?;
        Self::with_config(config)
    }

    /// Creates a new thread-local pool with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::{ThreadLocalPool, PoolConfig};
    ///
    /// let config: PoolConfig<i32> = PoolConfig::builder()
    ///     .capacity(100)
    ///     .alignment(64)
    ///     .build()
    ///     .unwrap();
    ///
    /// let pool = ThreadLocalPool::with_config(config).unwrap();
    /// ```
    pub fn with_config(config: PoolConfig<T>) -> Result<Self> {
        Ok(Self {
            pool: FixedPool::with_config(config)?,
            _marker: PhantomData,
        })
    }

    /// Allocates an object from the thread-local pool.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::ThreadLocalPool;
    ///
    /// let pool = ThreadLocalPool::new(10).unwrap();
    /// let handle = pool.allocate(42).unwrap();
    /// assert_eq!(*handle, 42);
    /// ```
    pub fn allocate(&self, value: T) -> Result<OwnedHandle<'_, T>> {
        self.pool.allocate(value)
    }

    /// Returns the total capacity of the pool.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.pool.capacity()
    }

    /// Returns the number of available (free) slots in the pool.
    #[inline]
    pub fn available(&self) -> usize {
        self.pool.available()
    }

    /// Returns the number of currently allocated objects.
    #[inline]
    pub fn allocated(&self) -> usize {
        self.pool.allocated()
    }

    /// Returns whether the pool is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.pool.is_full()
    }

    /// Returns whether the pool is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

// ThreadLocalPool is Send (can be moved to another thread) but not Sync
unsafe impl<T: Send> Send for ThreadLocalPool<T> {}
// Explicitly NOT implementing Sync - the !Sync marker prevents it

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_local_pool_basic() {
        let pool = ThreadLocalPool::<i32>::new(10).unwrap();

        let handle = pool.allocate(42).unwrap();
        assert_eq!(*handle, 42);
        assert_eq!(pool.allocated(), 1);

        drop(handle);
        assert_eq!(pool.allocated(), 0);
    }

    #[test]
    fn thread_local_pool_multiple_allocations() {
        let pool = ThreadLocalPool::new(5).unwrap();

        let _h1 = pool.allocate(1).unwrap();
        let _h2 = pool.allocate(2).unwrap();
        let _h3 = pool.allocate(3).unwrap();

        assert_eq!(pool.allocated(), 3);
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn thread_local_pool_capacity() {
        let pool = ThreadLocalPool::<i32>::new(3).unwrap();

        let _h1 = pool.allocate(1).unwrap();
        let _h2 = pool.allocate(2).unwrap();
        let _h3 = pool.allocate(3).unwrap();

        assert!(pool.is_full());

        let result = pool.allocate(4);
        assert!(result.is_err());
    }
}
