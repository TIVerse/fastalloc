//! Weak handle that doesn't prevent object return to pool.

use alloc::rc::Weak;
use core::fmt;

/// A weak handle to a pool-allocated object.
///
/// Weak handles do not contribute to the reference count and do not
/// prevent the object from being returned to the pool. They can be
/// upgraded to a `SharedHandle` if the object is still alive.
///
/// # Examples
///
/// ```rust
/// use fastalloc::{FixedPool, SharedHandle};
///
/// let pool = FixedPool::new(10).unwrap();
/// // This is a simplified example showing the concept
/// ```
pub struct WeakHandle<'pool, T> {
    inner: Weak<super::shared::SharedHandleInner<'pool, T>>,
}

impl<'pool, T> WeakHandle<'pool, T> {
    /// Creates a new weak handle from a weak reference.
    ///
    /// This is internal and should only be called by `SharedHandle::downgrade()`.
    #[inline]
    pub(crate) fn new(inner: Weak<super::shared::SharedHandleInner<'pool, T>>) -> Self {
        Self { inner }
    }
    
    /// Attempts to upgrade this weak handle to a shared handle.
    ///
    /// Returns `None` if the object has already been returned to the pool.
    #[inline]
    pub fn upgrade(&self) -> Option<super::SharedHandle<'pool, T>> {
        self.inner.upgrade().map(|inner| super::SharedHandle { inner })
    }
    
    /// Returns the number of strong references to the object, if it still exists.
    #[inline]
    pub fn strong_count(&self) -> usize {
        self.inner.strong_count()
    }
    
    /// Returns the number of weak references to the object.
    #[inline]
    pub fn weak_count(&self) -> usize {
        self.inner.weak_count()
    }
}

impl<'pool, T> Clone for WeakHandle<'pool, T> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
        }
    }
}

impl<'pool, T> fmt::Debug for WeakHandle<'pool, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeakHandle")
            .field("strong_count", &self.strong_count())
            .field("weak_count", &self.weak_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle::SharedHandle;
    use crate::pool::FixedPool;
    
    #[test]
    fn weak_handle_upgrade() {
        let pool = FixedPool::<i32>::new(10).unwrap();
        let handle = pool.allocate(42).unwrap();
        let index = handle.index();
        
        let shared = SharedHandle::new(&pool, index);
        let weak = shared.downgrade();
        
        assert_eq!(weak.strong_count(), 1);
        
        // Can upgrade while shared handle exists
        let upgraded = weak.upgrade();
        assert!(upgraded.is_some());
        assert_eq!(weak.strong_count(), 2);
        
        drop(shared);
        drop(upgraded);
        
        // Cannot upgrade after all strong references are gone
        let upgraded = weak.upgrade();
        assert!(upgraded.is_none());
        
        // Prevent double-free
        core::mem::forget(handle);
    }
    
    #[test]
    fn weak_handle_clone() {
        let pool = FixedPool::<i32>::new(10).unwrap();
        let handle = pool.allocate(42).unwrap();
        let index = handle.index();
        
        let shared = SharedHandle::new(&pool, index);
        let weak = shared.downgrade();
        let weak2 = weak.clone();
        
        assert_eq!(weak.weak_count(), weak2.weak_count());
        
        // Cleanup
        drop(shared);
        core::mem::forget(handle);
    }
}
