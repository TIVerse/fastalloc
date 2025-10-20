//! Shared handle with reference counting for pool-allocated objects.

use core::ops::Deref;
use core::fmt;
use alloc::rc::Rc;

/// A shared handle to a pool-allocated object with reference counting.
///
/// Multiple `SharedHandle` instances can point to the same object.
/// The object is returned to the pool only when the last handle is dropped.
///
/// # Examples
///
/// ```rust
/// use fastalloc::{FixedPool, SharedHandle};
///
/// let pool = FixedPool::<i32>::new(10).unwrap();
/// // Note: This is a simplified example. Actual implementation would need
/// // pool support for shared handles.
/// ```
///
/// # Note
///
/// This is currently a placeholder implementation. Full reference-counted
/// handles require additional pool infrastructure.
pub struct SharedHandle<'pool, T> {
    pub(crate) inner: Rc<SharedHandleInner<'pool, T>>,
}

pub(crate) struct SharedHandleInner<'pool, T> {
    pub(crate) pool: &'pool dyn super::owned::PoolInterface<T>,
    pub(crate) index: usize,
    pub(crate) _marker: core::marker::PhantomData<T>,
}

impl<'pool, T> SharedHandle<'pool, T> {
    /// Creates a new shared handle.
    ///
    /// This is internal and should only be called by pool implementations.
    #[inline]
    pub(crate) fn new(pool: &'pool dyn super::owned::PoolInterface<T>, index: usize) -> Self {
        Self {
            inner: Rc::new(SharedHandleInner {
                pool,
                index,
                _marker: core::marker::PhantomData,
            }),
        }
    }
    
    /// Returns the number of shared handles pointing to this object.
    #[inline]
    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }
    
    /// Returns the internal index of this handle.
    #[inline]
    pub fn index(&self) -> usize {
        self.inner.index
    }
    
    /// Creates a weak handle from this shared handle.
    pub fn downgrade(&self) -> super::WeakHandle<'pool, T> {
        super::WeakHandle::new(Rc::downgrade(&self.inner))
    }
}

impl<'pool, T> Clone for SharedHandle<'pool, T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<'pool, T> Deref for SharedHandle<'pool, T> {
    type Target = T;
    
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.pool.get(self.inner.index)
    }
}

impl<'pool, T> Drop for SharedHandleInner<'pool, T> {
    fn drop(&mut self) {
        // Return to pool when the last reference is dropped
        self.pool.return_to_pool(self.index);
    }
}

impl<'pool, T: fmt::Debug> fmt::Debug for SharedHandle<'pool, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedHandle")
            .field("index", &self.inner.index)
            .field("strong_count", &self.strong_count())
            .field("value", &**self)
            .finish()
    }
}

impl<'pool, T: fmt::Display> fmt::Display for SharedHandle<'pool, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'pool, T: PartialEq> PartialEq for SharedHandle<'pool, T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<'pool, T: Eq> Eq for SharedHandle<'pool, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::FixedPool;
    
    #[test]
    fn shared_handle_clone() {
        let pool = FixedPool::<i32>::new(10).unwrap();
        let handle = pool.allocate(42).unwrap();
        let index = handle.index();
        
        // Convert to shared handle (note: this bypasses normal pool lifecycle)
        let shared = SharedHandle::new(&pool, index);
        assert_eq!(shared.strong_count(), 1);
        
        let shared2 = shared.clone();
        assert_eq!(shared.strong_count(), 2);
        assert_eq!(shared2.strong_count(), 2);
        
        drop(shared2);
        assert_eq!(shared.strong_count(), 1);
        
        // Prevent double-free by forgetting the original handle
        core::mem::forget(handle);
    }
}
