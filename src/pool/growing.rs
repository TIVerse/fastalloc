//! Growing memory pool implementation.

use crate::allocator::{Allocator, FreeListAllocator};
use crate::config::PoolConfig;
use crate::error::{Error, Result};
use crate::handle::{OwnedHandle, PoolInterface};
use crate::traits::Poolable;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;

#[cfg(feature = "stats")]
use crate::stats::PoolStatistics;

/// A memory pool that can grow dynamically based on demand.
///
/// This pool starts with an initial capacity and can grow according to
/// a configurable growth strategy when it runs out of space.
///
/// # Examples
///
/// ```rust
/// use fastalloc::{GrowingPool, PoolConfig, GrowthStrategy};
///
/// let config = PoolConfig::builder()
///     .capacity(100)
///     .max_capacity(Some(1000))
///     .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
///     .build()
///     .unwrap();
///
/// let pool = GrowingPool::with_config(config).unwrap();
///
/// // Allocate objects - pool will grow as needed
/// let mut handles = Vec::new();
/// for i in 0..500 {
///     handles.push(pool.allocate(i).unwrap());
/// }
/// ```
///
/// # Performance
///
/// - Allocation: < 50ns per object (may spike during growth)
/// - Deallocation: < 15ns per object
/// - Growth causes temporary allocation spike
/// - Slight fragmentation possible with some growth strategies
pub struct GrowingPool<T> {
    /// Storage chunks
    storage: RefCell<Vec<Vec<MaybeUninit<T>>>>,
    /// Allocator for managing free slots
    allocator: RefCell<FreeListAllocator>,
    /// Current total capacity
    capacity: RefCell<usize>,
    /// Cumulative chunk sizes for fast O(log n) chunk lookup
    chunk_boundaries: RefCell<Vec<usize>>,
    /// Pool configuration
    config: PoolConfig<T>,
    /// Statistics collector
    #[cfg(feature = "stats")]
    stats: RefCell<crate::stats::StatisticsCollector>,
    /// Marker for lifetime and Send/Sync bounds
    _marker: PhantomData<T>,
}

impl<T: Poolable> GrowingPool<T> {
    /// Creates a new growing pool with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::{GrowingPool, PoolConfig, GrowthStrategy};
    ///
    /// let config: PoolConfig<i32> = PoolConfig::builder()
    ///     .capacity(100)
    ///     .growth_strategy(GrowthStrategy::Linear { amount: 50 })
    ///     .build()
    ///     .unwrap();
    ///
    /// let pool = GrowingPool::with_config(config).unwrap();
    /// ```
    pub fn with_config(config: PoolConfig<T>) -> Result<Self> {
        let capacity = config.capacity();

        // Allocate initial storage chunk
        let mut storage_chunk = Vec::with_capacity(capacity);
        storage_chunk.resize_with(capacity, MaybeUninit::uninit);

        let storage = vec![storage_chunk];

        let pool = Self {
            storage: RefCell::new(storage),
            allocator: RefCell::new(FreeListAllocator::new(capacity)),
            capacity: RefCell::new(capacity),
            chunk_boundaries: RefCell::new(vec![capacity]),
            config,
            #[cfg(feature = "stats")]
            stats: RefCell::new(crate::stats::StatisticsCollector::new(capacity)),
            _marker: PhantomData,
        };

        Ok(pool)
    }

    /// Grows the pool by allocating an additional chunk of memory.
    fn grow(&self) -> Result<()> {
        let growth_amount = self
            .config
            .growth_strategy()
            .compute_growth(*self.capacity.borrow());

        if growth_amount == 0 {
            return Err(Error::PoolExhausted {
                capacity: *self.capacity.borrow(),
                allocated: *self.capacity.borrow() - self.allocator.borrow().available(),
            });
        }

        let current_capacity = *self.capacity.borrow();
        let new_capacity = current_capacity + growth_amount;

        // Check max capacity constraint
        if let Some(max) = self.config.max_capacity() {
            if new_capacity > max {
                return Err(Error::MaxCapacityExceeded {
                    current: current_capacity,
                    requested: new_capacity,
                    max,
                });
            }
        }

        // Allocate new storage chunk
        let mut new_chunk = Vec::with_capacity(growth_amount);
        new_chunk.resize_with(growth_amount, MaybeUninit::uninit);

        self.storage.borrow_mut().push(new_chunk);
        self.allocator.borrow_mut().extend(growth_amount);
        *self.capacity.borrow_mut() = new_capacity;
        self.chunk_boundaries.borrow_mut().push(new_capacity);

        #[cfg(feature = "stats")]
        self.stats.borrow_mut().record_growth(new_capacity);

        Ok(())
    }

    /// Allocates an object from the pool with the given initial value.
    ///
    /// If the pool is full, it will attempt to grow according to its growth strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::{GrowingPool, PoolConfig, GrowthStrategy};
    ///
    /// let config = PoolConfig::builder()
    ///     .capacity(2)
    ///     .growth_strategy(GrowthStrategy::Linear { amount: 2 })
    ///     .build()
    ///     .unwrap();
    ///
    /// let pool = GrowingPool::with_config(config).unwrap();
    ///
    /// let h1 = pool.allocate(1).unwrap();
    /// let h2 = pool.allocate(2).unwrap();
    /// // Pool will grow automatically
    /// let h3 = pool.allocate(3).unwrap();
    /// ```
    pub fn allocate(&self, mut value: T) -> Result<OwnedHandle<'_, T>> {
        // Try to allocate a slot
        let index = {
            let mut allocator = self.allocator.borrow_mut();
            if let Some(idx) = allocator.allocate() {
                idx
            } else {
                // Drop the borrow before growing
                drop(allocator);

                // Pool is full, try to grow
                self.grow()?;

                // Try again after growth
                self.allocator
                    .borrow_mut()
                    .allocate()
                    .ok_or_else(|| Error::PoolExhausted {
                        capacity: *self.capacity.borrow(),
                        allocated: *self.capacity.borrow(),
                    })?
            }
        };

        #[cfg(feature = "stats")]
        self.stats.borrow_mut().record_allocation();

        // Call on_acquire hook
        value.on_acquire();

        // Find which chunk and offset, then write the value
        {
            let mut storage = self.storage.borrow_mut();
            let mut remaining = index;
            let mut found = false;

            for chunk in storage.iter_mut() {
                if remaining < chunk.len() {
                    chunk[remaining].write(value);
                    found = true;
                    break;
                }
                remaining -= chunk.len();
            }

            if !found {
                panic!("Index out of bounds: {}", index);
            }
        }

        Ok(OwnedHandle::new(self, index))
    }

    /// Internal allocation method that returns just the index.
    ///
    /// This is used by thread-safe wrappers to allocate without creating a handle.
    pub(crate) fn allocate_internal(&mut self, mut value: T) -> Result<usize> {
        // Try to allocate a slot
        let index = {
            let mut allocator = self.allocator.borrow_mut();
            if let Some(idx) = allocator.allocate() {
                idx
            } else {
                // Drop the borrow before growing
                drop(allocator);

                // Pool is full, try to grow
                self.grow()?;

                // Try again after growth
                self.allocator
                    .borrow_mut()
                    .allocate()
                    .ok_or_else(|| Error::PoolExhausted {
                        capacity: *self.capacity.borrow(),
                        allocated: *self.capacity.borrow(),
                    })?
            }
        };

        #[cfg(feature = "stats")]
        self.stats.borrow_mut().record_allocation();

        // Call on_acquire hook
        value.on_acquire();

        // Find which chunk and offset, then write the value
        {
            let mut storage = self.storage.borrow_mut();
            let mut remaining = index;
            let mut found = false;

            for chunk in storage.iter_mut() {
                if remaining < chunk.len() {
                    chunk[remaining].write(value);
                    found = true;
                    break;
                }
                remaining -= chunk.len();
            }

            if !found {
                panic!("Index out of bounds: {}", index);
            }
        }

        Ok(index)
    }

    /// Converts a flat index to chunk index and offset within that chunk.
    /// Returns (chunk_index, offset_within_chunk)
    /// Uses cached chunk boundaries for fast O(log n) binary search lookup.
    #[inline]
    fn compute_chunk_location(&self, index: usize) -> (usize, usize) {
        let boundaries = self.chunk_boundaries.borrow();
        
        // Binary search to find the chunk
        let chunk_idx = match boundaries.binary_search(&(index + 1)) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };
        
        // Compute offset within chunk
        let offset = if chunk_idx == 0 {
            index
        } else {
            index - boundaries[chunk_idx - 1]
        };
        
        (chunk_idx, offset)
    }

    /// Returns the total capacity of the pool.
    #[inline]
    pub fn capacity(&self) -> usize {
        *self.capacity.borrow()
    }

    /// Returns the number of available (free) slots in the pool.
    #[inline]
    pub fn available(&self) -> usize {
        self.allocator.borrow().available()
    }

    /// Returns the number of currently allocated objects.
    #[inline]
    pub fn allocated(&self) -> usize {
        self.capacity() - self.available()
    }

    /// Returns whether the pool is full (no available slots and cannot grow).
    #[inline]
    pub fn is_full(&self) -> bool {
        self.allocator.borrow().is_full() && !self.can_grow()
    }

    /// Returns whether the pool is empty (all slots available).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.allocator.borrow().is_empty()
    }

    /// Returns whether the pool can grow further.
    #[inline]
    pub fn can_grow(&self) -> bool {
        if !self.config.growth_strategy().allows_growth() {
            return false;
        }

        if let Some(max) = self.config.max_capacity() {
            self.capacity() < max
        } else {
            true
        }
    }

    /// Gets a reference to an object at the given index.
    ///
    /// # Safety
    ///
    /// This is internal and should only be called with valid allocated indices.
    #[inline]
    pub(crate) fn get(&self, index: usize) -> &T {
        let (chunk_idx, offset) = self.compute_chunk_location(index);
        let storage = self.storage.borrow();
        // Safety: index is valid and initialized by allocate()
        // We extend the lifetime beyond the borrow - safe because pool owns the data
        unsafe {
            let ptr = storage.as_ptr();
            let chunk = &*ptr.add(chunk_idx);
            &*chunk.as_ptr().add(offset).cast::<T>()
        }
    }

    /// Gets a mutable reference to an object at the given index.
    ///
    /// # Safety
    ///
    /// This is internal and should only be called with valid allocated indices.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_mut(&self, index: usize) -> &mut T {
        let (chunk_idx, offset) = self.compute_chunk_location(index);
        let storage = self.storage.borrow_mut();
        // Safety: index is valid and initialized by allocate()
        // We extend the lifetime beyond the borrow - safe because pool owns the data
        unsafe {
            let ptr = storage.as_ptr() as *mut Vec<MaybeUninit<T>>;
            let chunk = &mut *ptr.add(chunk_idx);
            &mut *chunk.as_mut_ptr().add(offset).cast::<T>()
        }
    }

    /// Returns an object to the pool.
    pub(crate) fn return_to_pool(&self, index: usize) {
        let (chunk_idx, offset) = self.compute_chunk_location(index);

        // Get the value and call on_release
        let mut storage = self.storage.borrow_mut();

        unsafe {
            let value_ptr = storage[chunk_idx][offset].as_mut_ptr();
            (*value_ptr).on_release();
            ptr::drop_in_place(value_ptr);
        }

        // Mark the slot as free
        self.allocator.borrow_mut().free(index);

        #[cfg(feature = "stats")]
        self.stats.borrow_mut().record_deallocation();
    }

    /// Get current pool statistics.
    #[cfg(feature = "stats")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stats")))]
    pub fn statistics(&self) -> PoolStatistics {
        let mut stats = self.stats.borrow().snapshot();
        stats.current_usage = self.allocated();
        stats.capacity = self.capacity();
        stats
    }

    /// Reset statistics counters.
    #[cfg(feature = "stats")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stats")))]
    pub fn reset_statistics(&self) {
        self.stats.borrow_mut().reset();
    }
}

impl<T: Poolable> PoolInterface<T> for GrowingPool<T> {
    #[inline]
    fn get(&self, index: usize) -> &T {
        self.get(index)
    }

    #[inline]
    fn get_mut(&self, index: usize) -> &mut T {
        self.get_mut(index)
    }

    #[inline]
    fn return_to_pool(&self, index: usize) {
        self.return_to_pool(index)
    }
}

unsafe impl<T: Send> Send for GrowingPool<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GrowthStrategy;

    #[test]
    fn new_pool() {
        let config = PoolConfig::builder()
            .capacity(100)
            .growth_strategy(GrowthStrategy::Linear { amount: 50 })
            .build()
            .unwrap();

        let pool = GrowingPool::<i32>::with_config(config).unwrap();
        assert_eq!(pool.capacity(), 100);
        assert_eq!(pool.available(), 100);
    }

    #[test]
    fn pool_grows_on_demand() {
        let config = PoolConfig::builder()
            .capacity(2)
            .growth_strategy(GrowthStrategy::Linear { amount: 2 })
            .build()
            .unwrap();

        let pool = GrowingPool::with_config(config).unwrap();

        let _h1 = pool.allocate(1).unwrap();
        let _h2 = pool.allocate(2).unwrap();
        assert_eq!(pool.capacity(), 2);

        // This should trigger growth
        let _h3 = pool.allocate(3).unwrap();
        assert_eq!(pool.capacity(), 4);
    }

    #[test]
    fn respects_max_capacity() {
        let config = PoolConfig::builder()
            .capacity(2)
            .max_capacity(Some(4))
            .growth_strategy(GrowthStrategy::Linear { amount: 2 })
            .build()
            .unwrap();

        let pool = GrowingPool::with_config(config).unwrap();

        let _h1 = pool.allocate(1).unwrap();
        let _h2 = pool.allocate(2).unwrap();
        let _h3 = pool.allocate(3).unwrap(); // Grows to 4
        let _h4 = pool.allocate(4).unwrap();

        assert_eq!(pool.capacity(), 4);

        // Should fail - cannot grow beyond max
        let result = pool.allocate(5);
        assert!(matches!(result, Err(Error::MaxCapacityExceeded { .. })));
    }
}
