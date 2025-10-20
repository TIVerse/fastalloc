//! Owned handle that exclusively owns a pool-allocated object.

use core::ops::{Deref, DerefMut};
use core::fmt;

/// An owned handle to a pool-allocated object.
///
/// This handle provides exclusive access to an object and automatically
/// returns it to the pool when dropped (RAII pattern).
///
/// # Examples
///
/// ```rust
/// use fastalloc::FixedPool;
///
/// let pool = FixedPool::new(10).unwrap();
/// let mut handle = pool.allocate(42).unwrap();
///
/// // Read access
/// assert_eq!(*handle, 42);
///
/// // Write access
/// *handle = 100;
/// assert_eq!(*handle, 100);
///
/// // Automatically returned to pool when dropped
/// ```
pub struct OwnedHandle<'pool, T> {
    pool: &'pool dyn PoolInterface<T>,
    index: usize,
    _marker: core::marker::PhantomData<T>,
}

/// Internal trait for pool operations needed by handles.
///
/// This trait is used internally to allow handles to work with different
/// pool types without exposing implementation details.
pub trait PoolInterface<T> {
    #[doc(hidden)]
    fn get(&self, index: usize) -> &T;
    #[doc(hidden)]
    fn get_mut(&self, index: usize) -> &mut T;
    #[doc(hidden)]
    fn return_to_pool(&self, index: usize);
}

impl<'pool, T> OwnedHandle<'pool, T> {
    /// Creates a new owned handle.
    ///
    /// This is internal and should only be called by pool implementations.
    #[inline]
    pub(crate) fn new(pool: &'pool dyn PoolInterface<T>, index: usize) -> Self {
        Self {
            pool,
            index,
            _marker: core::marker::PhantomData,
        }
    }
    
    /// Returns the internal index of this handle.
    ///
    /// This is useful for debugging but should not be relied upon for
    /// application logic.
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }
}

impl<'pool, T> Deref for OwnedHandle<'pool, T> {
    type Target = T;
    
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.pool.get(self.index)
    }
}

impl<'pool, T> DerefMut for OwnedHandle<'pool, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.pool.get_mut(self.index)
    }
}

impl<'pool, T> Drop for OwnedHandle<'pool, T> {
    fn drop(&mut self) {
        self.pool.return_to_pool(self.index);
    }
}

impl<'pool, T: fmt::Debug> fmt::Debug for OwnedHandle<'pool, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedHandle")
            .field("index", &self.index)
            .field("value", &**self)
            .finish()
    }
}

impl<'pool, T: fmt::Display> fmt::Display for OwnedHandle<'pool, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

// Implement PartialEq by comparing the contained values
impl<'pool, T: PartialEq> PartialEq for OwnedHandle<'pool, T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<'pool, T: Eq> Eq for OwnedHandle<'pool, T> {}

impl<'pool, T: PartialOrd> PartialOrd for OwnedHandle<'pool, T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<'pool, T: Ord> Ord for OwnedHandle<'pool, T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

// Implement common traits for FixedPool to satisfy PoolInterface
impl<T: crate::traits::Poolable> super::owned::PoolInterface<T> for crate::pool::FixedPool<T> {
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

#[cfg(test)]
mod tests {
    use crate::pool::FixedPool;
    
    #[test]
    fn handle_deref() {
        let pool = FixedPool::new(10).unwrap();
        let handle = pool.allocate(42).unwrap();
        
        assert_eq!(*handle, 42);
    }
    
    #[test]
    fn handle_deref_mut() {
        let pool = FixedPool::new(10).unwrap();
        let mut handle = pool.allocate(10).unwrap();
        
        *handle = 20;
        assert_eq!(*handle, 20);
    }
    
    #[test]
    fn handle_drop() {
        let pool = FixedPool::new(10).unwrap();
        
        {
            let _handle = pool.allocate(42).unwrap();
            assert_eq!(pool.allocated(), 1);
        }
        
        assert_eq!(pool.allocated(), 0);
    }
    
    #[test]
    fn handle_equality() {
        let pool = FixedPool::new(10).unwrap();
        let h1 = pool.allocate(42).unwrap();
        let h2 = pool.allocate(42).unwrap();
        let h3 = pool.allocate(99).unwrap();
        
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
