//! Thread-safe memory pool implementations.

use crate::config::PoolConfig;
use crate::error::Result;
use core::ops::{Deref, DerefMut};

#[cfg(not(feature = "parking_lot"))]
use std::sync::{Arc, Mutex};

#[cfg(feature = "parking_lot")]
use parking_lot::Mutex;
#[cfg(feature = "parking_lot")]
use std::sync::Arc;

/// Handle for thread-safe pool allocations.
///
/// This handle holds a reference to the pool and automatically returns
/// the object when dropped, with proper locking.
/// 
/// Performance note: This handle caches the pointer to avoid locking
/// on every dereference operation, only locking during allocation and deallocation.
pub struct ThreadSafeHandle<T: crate::traits::Poolable> {
    pool: Arc<Mutex<crate::pool::GrowingPool<T>>>,
    index: usize,
    /// Cached pointer to the value for lock-free deref
    cached_ptr: *mut T,
}

impl<T: crate::traits::Poolable> Deref for ThreadSafeHandle<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // Safety: The cached pointer is valid for the lifetime of this handle.
        // The pool storage is stable (won't move) and this handle has exclusive
        // ownership of the slot via allocator tracking.
        unsafe { &*self.cached_ptr }
    }
}

impl<T: crate::traits::Poolable> DerefMut for ThreadSafeHandle<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: The cached pointer is valid for the lifetime of this handle.
        // We have &mut self so we have exclusive access to the handle.
        unsafe { &mut *self.cached_ptr }
    }
}

impl<T: crate::traits::Poolable> Drop for ThreadSafeHandle<T> {
    fn drop(&mut self) {
        #[cfg(not(feature = "parking_lot"))]
        let pool = self.pool.lock().unwrap();
        #[cfg(feature = "parking_lot")]
        let pool = self.pool.lock();

        pool.return_to_pool(self.index);
    }
}

// Safety: ThreadSafeHandle can be sent across threads if T is Send
// The raw pointer is only accessed through the handle which ensures exclusive access
unsafe impl<T: crate::traits::Poolable + Send> Send for ThreadSafeHandle<T> {}

// Note: ThreadSafeHandle is intentionally NOT Sync because it contains a raw pointer
// and provides mutable access through DerefMut. Each handle should be owned by a single thread.

/// A thread-safe memory pool using locks for synchronization.
///
/// This pool can be safely shared across threads and used concurrently.
/// It uses `Mutex` for synchronization (or `parking_lot::Mutex` if the
/// feature is enabled for better performance).
///
/// # Examples
///
/// ```rust
/// use fastalloc::ThreadSafePool;
/// use std::sync::Arc;
///
/// let pool = Arc::new(ThreadSafePool::<i32>::new(1000).unwrap());
///
/// // Allocate from the pool
/// let handle1 = pool.allocate(42).unwrap();
/// assert_eq!(*handle1, 42);
/// drop(handle1);
///
/// // Can be shared across threads
/// let pool_clone = Arc::clone(&pool);
/// let handle2 = pool_clone.allocate(100).unwrap();
/// assert_eq!(*handle2, 100);
/// ```
///
/// # Performance
///
/// - Allocation: < 100ns with moderate contention (typical)
/// - Higher latency under heavy contention
/// - Use `ThreadLocalPool` for single-threaded performance
pub struct ThreadSafePool<T> {
    inner: Arc<Mutex<crate::pool::GrowingPool<T>>>,
}

impl<T: crate::traits::Poolable> ThreadSafePool<T> {
    /// Creates a new thread-safe pool with the specified capacity.
    pub fn new(capacity: usize) -> Result<Self> {
        let config = PoolConfig::builder().capacity(capacity).build()?;
        Self::with_config(config)
    }

    /// Creates a new thread-safe pool with the specified configuration.
    pub fn with_config(config: PoolConfig<T>) -> Result<Self> {
        let pool = crate::pool::GrowingPool::with_config(config)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(pool)),
        })
    }

    /// Allocates an object from the pool.
    ///
    /// This method acquires a lock and may block if another thread is
    /// currently using the pool.
    pub fn allocate(&self, value: T) -> Result<ThreadSafeHandle<T>> {
        #[cfg(not(feature = "parking_lot"))]
        let mut pool = self.inner.lock().unwrap();

        #[cfg(feature = "parking_lot")]
        let mut pool = self.inner.lock();

        // Allocate using the internal pool API
        let index = pool.allocate_internal(value)?;
        
        // Cache the pointer for lock-free deref
        let cached_ptr = pool.get_mut(index) as *mut T;

        Ok(ThreadSafeHandle {
            pool: Arc::clone(&self.inner),
            index,
            cached_ptr,
        })
    }

    /// Returns the current capacity of the pool.
    pub fn capacity(&self) -> usize {
        #[cfg(not(feature = "parking_lot"))]
        let pool = self.inner.lock().unwrap();

        #[cfg(feature = "parking_lot")]
        let pool = self.inner.lock();

        pool.capacity()
    }

    /// Returns the number of available slots.
    pub fn available(&self) -> usize {
        #[cfg(not(feature = "parking_lot"))]
        let pool = self.inner.lock().unwrap();

        #[cfg(feature = "parking_lot")]
        let pool = self.inner.lock();

        pool.available()
    }

    /// Returns the number of currently allocated objects.
    pub fn allocated(&self) -> usize {
        #[cfg(not(feature = "parking_lot"))]
        let pool = self.inner.lock().unwrap();

        #[cfg(feature = "parking_lot")]
        let pool = self.inner.lock();

        pool.allocated()
    }
}

impl<T> Clone for ThreadSafePool<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// ThreadSafePool is Send + Sync when T is Send
unsafe impl<T: Send> Send for ThreadSafePool<T> {}
unsafe impl<T: Send> Sync for ThreadSafePool<T> {}

/// A lock-free memory pool using atomic operations.
///
/// This pool provides better performance under high contention compared
/// to `ThreadSafePool` by avoiding locks. Requires the `lock-free` feature.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "lock-free")]
/// # {
/// use fastalloc::LockFreePool;
/// use std::sync::Arc;
/// use std::thread;
///
/// let pool = Arc::new(LockFreePool::<i32>::with_initializer(1000, || 0).unwrap());
///
/// let mut handles = vec![];
/// for i in 0..8 {
///     let pool_clone = Arc::clone(&pool);
///     handles.push(thread::spawn(move || {
///         for _j in 0..10 {
///             if let Some(obj) = pool_clone.try_allocate() {
///                 pool_clone.return_object(obj);
///             }
///         }
///     }));
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// # }
/// ```
#[cfg(feature = "lock-free")]
#[cfg_attr(docsrs, doc(cfg(feature = "lock-free")))]
pub struct LockFreePool<T> {
    inner: Arc<crossbeam::queue::SegQueue<Box<T>>>,
    capacity: std::sync::atomic::AtomicUsize,
}

#[cfg(feature = "lock-free")]
impl<T> LockFreePool<T> {
    /// Creates a new lock-free pool with the specified capacity.
    ///
    /// Note: The current implementation is a simplified version.
    /// A full production implementation would use a more sophisticated
    /// lock-free data structure.
    pub fn new(capacity: usize) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(crossbeam::queue::SegQueue::new()),
            capacity: std::sync::atomic::AtomicUsize::new(capacity),
        })
    }

    /// Pre-populates the pool with objects created by the initializer.
    pub fn with_initializer<F>(capacity: usize, mut init: F) -> Result<Self>
    where
        F: FnMut() -> T,
    {
        let pool = Self::new(capacity)?;
        for _ in 0..capacity {
            pool.inner.push(Box::new(init()));
        }
        Ok(pool)
    }

    /// Attempts to allocate an object from the pool.
    ///
    /// If the pool is empty, this will fail. Unlike other pool types,
    /// this simplified lock-free implementation does not automatically grow.
    pub fn try_allocate(&self) -> Option<Box<T>> {
        self.inner.pop()
    }

    /// Returns an object to the pool.
    pub fn return_object(&self, object: Box<T>) {
        self.inner.push(object);
    }
}

#[cfg(feature = "lock-free")]
impl<T> Clone for LockFreePool<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            capacity: std::sync::atomic::AtomicUsize::new(
                self.capacity.load(std::sync::atomic::Ordering::Relaxed),
            ),
        }
    }
}

#[cfg(feature = "lock-free")]
unsafe impl<T: Send> Send for LockFreePool<T> {}

#[cfg(feature = "lock-free")]
unsafe impl<T: Send> Sync for LockFreePool<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_safe_pool_basic() {
        let pool = ThreadSafePool::<i32>::new(10).unwrap();

        let handle = pool.allocate(42).unwrap();
        assert_eq!(*handle, 42);
    }

    #[test]
    fn thread_safe_pool_concurrent() {
        use std::thread;

        let pool = Arc::new(ThreadSafePool::<i32>::new(100).unwrap());

        let mut handles = vec![];
        for i in 0..4 {
            let pool_clone = Arc::clone(&pool);
            handles.push(thread::spawn(move || {
                let _h = pool_clone.allocate(i).unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[cfg(feature = "lock-free")]
    #[test]
    fn lock_free_pool_basic() {
        let pool = LockFreePool::<i32>::with_initializer(10, || 0).unwrap();

        let obj = pool.try_allocate();
        assert!(obj.is_some());

        pool.return_object(obj.unwrap());
    }
}
