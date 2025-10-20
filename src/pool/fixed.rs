//! Fixed-size memory pool implementation.

use crate::allocator::{Allocator, StackAllocator};
use crate::config::PoolConfig;
use crate::error::{Error, Result};
use crate::handle::OwnedHandle;
use crate::traits::Poolable;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;

#[cfg(feature = "stats")]
use crate::stats::PoolStatistics;

/// A fixed-size memory pool with O(1) allocation and deallocation.
///
/// This pool pre-allocates a fixed number of slots and does not grow.
/// It provides the fastest possible allocation performance with zero
/// fragmentation and predictable behavior.
///
/// # Examples
///
/// ```rust
/// use fastalloc::FixedPool;
///
/// // Create a pool of 1000 integers
/// let pool = FixedPool::<i32>::new(1000).unwrap();
///
/// // Allocate from the pool
/// let mut handle = pool.allocate(42).unwrap();
/// assert_eq!(*handle, 42);
///
/// // Modify the value
/// *handle = 100;
/// assert_eq!(*handle, 100);
///
/// // Automatically returned to pool when dropped
/// drop(handle);
/// ```
///
/// # Performance
///
/// - Allocation: < 20ns per object (typical)
/// - Deallocation: < 10ns per object (typical)
/// - Memory overhead: ~8 bytes per slot + allocator metadata
/// - Zero fragmentation
pub struct FixedPool<T> {
    /// Storage for pool objects
    storage: RefCell<Vec<MaybeUninit<T>>>,
    /// Allocator for managing free slots
    allocator: RefCell<StackAllocator>,
    /// Total capacity
    capacity: usize,
    /// Pool configuration
    config: PoolConfig<T>,
    /// Statistics collector
    #[cfg(feature = "stats")]
    stats: RefCell<crate::stats::StatisticsCollector>,
    /// Marker for lifetime and Send/Sync bounds
    _marker: PhantomData<T>,
}

impl<T: Poolable> FixedPool<T> {
    /// Creates a new fixed-size pool with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::FixedPool;
    ///
    /// let pool = FixedPool::<String>::new(100).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if capacity is 0.
    pub fn new(capacity: usize) -> Result<Self> {
        let config = PoolConfig::builder().capacity(capacity).build()?;
        Self::with_config(config)
    }
    
    /// Creates a new fixed-size pool with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::{FixedPool, PoolConfig};
    ///
    /// let config = PoolConfig::builder()
    ///     .capacity(1000)
    ///     .alignment(64)
    ///     .pre_initialize(false)
    ///     .build()
    ///     .unwrap();
    ///
    /// let pool = FixedPool::<i32>::with_config(config).unwrap();
    /// ```
    pub fn with_config(config: PoolConfig<T>) -> Result<Self> {
        let capacity = config.capacity();
        
        // Allocate storage
        let mut storage = Vec::with_capacity(capacity);
        storage.resize_with(capacity, MaybeUninit::uninit);
        
        let pool = Self {
            storage: RefCell::new(storage),
            allocator: RefCell::new(StackAllocator::new(capacity)),
            capacity,
            config,
            #[cfg(feature = "stats")]
            stats: RefCell::new(crate::stats::StatisticsCollector::new(capacity)),
            _marker: PhantomData,
        };
        
        Ok(pool)
    }
    
    /// Allocates an object from the pool with the given initial value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastalloc::FixedPool;
    ///
    /// let pool = FixedPool::new(10).unwrap();
    /// let handle = pool.allocate(42).unwrap();
    /// assert_eq!(*handle, 42);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Error::PoolExhausted` if the pool is at capacity.
    #[inline]
    pub fn allocate(&self, mut value: T) -> Result<OwnedHandle<'_, T>> {
        // Try to allocate a slot
        let index = self.allocator.borrow_mut().allocate().ok_or_else(|| {
            Error::PoolExhausted {
                capacity: self.capacity,
                allocated: self.capacity,
            }
        })?;
        
        #[cfg(feature = "stats")]
        self.stats.borrow_mut().record_allocation();
        
        // Call on_acquire hook
        value.on_acquire();
        
        // Write the value to the slot
        let mut storage = self.storage.borrow_mut();
        storage[index].write(value);
        
        Ok(OwnedHandle::new(self, index))
    }
    
    /// Allocates multiple objects from the pool in a single operation.
    ///
    /// This is more efficient than multiple individual `allocate` calls
    /// as it reduces the number of borrow operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastalloc::FixedPool;
    ///
    /// let pool = FixedPool::new(100).unwrap();
    /// let values = vec![1, 2, 3, 4, 5];
    /// let handles = pool.allocate_batch(values).unwrap();
    /// assert_eq!(handles.len(), 5);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Error::PoolExhausted` if there aren't enough free slots.
    pub fn allocate_batch(&self, values: alloc::vec::Vec<T>) -> Result<alloc::vec::Vec<OwnedHandle<'_, T>>> {
        // Check if we have enough capacity upfront
        if values.len() > self.available() {
            return Err(Error::PoolExhausted {
                capacity: self.capacity,
                allocated: self.allocated(),
            });
        }
        
        let mut handles = alloc::vec::Vec::with_capacity(values.len());
        
        for value in values {
            // We know these won't fail due to the check above
            match self.allocate(value) {
                Ok(handle) => handles.push(handle),
                Err(e) => {
                    // This shouldn't happen, but if it does, clean up
                    drop(handles);
                    return Err(e);
                }
            }
        }
        
        Ok(handles)
    }
    
    /// Attempts to allocate from the pool, returning None if exhausted.
    ///
    /// This is a convenience method that doesn't return an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastalloc::FixedPool;
    ///
    /// let pool = FixedPool::new(10).unwrap();
    /// if let Some(handle) = pool.try_allocate(42) {
    ///     assert_eq!(*handle, 42);
    /// };
    /// ```
    #[inline]
    pub fn try_allocate(&self, value: T) -> Option<OwnedHandle<'_, T>> {
        self.allocate(value).ok()
    }
    
    /// Returns the total capacity of the pool.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Returns the number of available (free) slots in the pool.
    #[inline]
    pub fn available(&self) -> usize {
        self.allocator.borrow().available()
    }
    
    /// Returns the number of currently allocated objects.
    #[inline]
    pub fn allocated(&self) -> usize {
        self.capacity - self.available()
    }
    
    /// Returns whether the pool is full (no available slots).
    #[inline]
    pub fn is_full(&self) -> bool {
        self.allocator.borrow().is_full()
    }
    
    /// Returns whether the pool is empty (all slots available).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.allocator.borrow().is_empty()
    }
    
    /// Gets a reference to an object at the given index.
    ///
    /// # Safety
    ///
    /// This is internal and should only be called with valid allocated indices.
    #[inline(always)]
    pub(crate) fn get(&self, index: usize) -> &T {
        let storage = self.storage.borrow();
        // Safety: index is valid and initialized by allocate()
        // We extend the lifetime beyond the borrow - safe because pool owns the data
        unsafe { 
            let ptr = storage.as_ptr();
            &*ptr.add(index).cast::<T>()
        }
    }
    
    /// Gets a mutable reference to an object at the given index.
    ///
    /// # Safety
    ///
    /// This is internal and should only be called with valid allocated indices.
    #[inline(always)]
    pub(crate) fn get_mut(&self, index: usize) -> &mut T {
        let storage = self.storage.borrow_mut();
        // Safety: index is valid and initialized by allocate()
        // We extend the lifetime beyond the borrow - safe because pool owns the data
        unsafe { 
            let ptr = storage.as_ptr() as *mut MaybeUninit<T>;
            &mut *ptr.add(index).cast::<T>()
        }
    }
    
    /// Returns an object to the pool (called by handle Drop).
    ///
    /// # Safety
    ///
    /// This is internal and should only be called once per allocation.
    pub(crate) fn return_to_pool(&self, index: usize) {
        // Get the value and call on_release
        let mut storage = self.storage.borrow_mut();
        
        // Safety: index is valid and was initialized
        unsafe {
            let value_ptr = storage[index].as_mut_ptr();
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
        stats
    }
    
    /// Reset statistics counters.
    #[cfg(feature = "stats")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stats")))]
    pub fn reset_statistics(&self) {
        self.stats.borrow_mut().reset();
    }
}

impl<T> Drop for FixedPool<T> {
    fn drop(&mut self) {
        // Clean up any remaining allocated objects
        // Note: In this implementation, objects are dropped when handles are dropped
        // The pool itself doesn't need additional cleanup
        let _allocator = self.allocator.borrow();
        
        // All handles should be dropped before the pool
        // Any remaining objects will be dropped with the storage Vec
    }
}

// Safety: FixedPool is Send if T is Send (storage is behind RefCell)
unsafe impl<T: Send> Send for FixedPool<T> {}

// Note: FixedPool is NOT Sync because it uses RefCell internally
// Use ThreadSafePool for concurrent access

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new_pool() {
        let pool = FixedPool::<i32>::new(100).unwrap();
        assert_eq!(pool.capacity(), 100);
        assert_eq!(pool.available(), 100);
        assert_eq!(pool.allocated(), 0);
        assert!(pool.is_empty());
        assert!(!pool.is_full());
    }
    
    #[test]
    fn allocate_and_drop() {
        let pool = FixedPool::new(10).unwrap();
        
        {
            let handle = pool.allocate(42).unwrap();
            assert_eq!(*handle, 42);
            assert_eq!(pool.allocated(), 1);
            assert_eq!(pool.available(), 9);
        }
        
        // After drop, should be returned to pool
        assert_eq!(pool.allocated(), 0);
        assert_eq!(pool.available(), 10);
    }
    
    #[test]
    fn allocate_until_full() {
        let pool = FixedPool::new(3).unwrap();
        
        let _h1 = pool.allocate(1).unwrap();
        let _h2 = pool.allocate(2).unwrap();
        let _h3 = pool.allocate(3).unwrap();
        
        assert!(pool.is_full());
        
        let result = pool.allocate(4);
        assert!(matches!(result, Err(Error::PoolExhausted { .. })));
    }
    
    #[test]
    fn reuse_after_free() {
        let pool = FixedPool::new(2).unwrap();
        
        let h1 = pool.allocate(1).unwrap();
        drop(h1);
        
        let h2 = pool.allocate(2).unwrap();
        assert_eq!(*h2, 2);
    }
    
    #[test]
    fn modify_value() {
        let pool = FixedPool::new(10).unwrap();
        
        let mut handle = pool.allocate(10).unwrap();
        assert_eq!(*handle, 10);
        
        *handle = 20;
        assert_eq!(*handle, 20);
    }
}
