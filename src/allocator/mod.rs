//! Internal allocation strategies for managing pool memory.

mod stack;
mod freelist;
mod bitmap;

pub(crate) use stack::StackAllocator;
pub(crate) use freelist::FreeListAllocator;
#[allow(unused)]
pub(crate) use bitmap::BitmapAllocator;

/// Trait for internal allocation strategies.
///
/// This trait is used internally by pool implementations to manage
/// which slots are available for allocation.
pub(crate) trait Allocator {
    /// Allocates a slot index, returning the index if successful.
    fn allocate(&mut self) -> Option<usize>;
    
    /// Frees a previously allocated slot.
    fn free(&mut self, index: usize);
    
    /// Returns the number of available slots.
    fn available(&self) -> usize;
    
    /// Returns the total capacity.
    fn capacity(&self) -> usize;
    
    /// Returns whether the allocator is full.
    #[inline]
    fn is_full(&self) -> bool {
        self.available() == 0
    }
    
    /// Returns whether the allocator is empty (all slots free).
    #[inline]
    fn is_empty(&self) -> bool {
        self.available() == self.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_allocator<A: Allocator>(mut allocator: A) {
        let capacity = allocator.capacity();
        assert_eq!(allocator.available(), capacity);
        assert!(allocator.is_empty());
        assert!(!allocator.is_full());
        
        // Allocate all slots
        let mut indices = Vec::new();
        for _ in 0..capacity {
            let idx = allocator.allocate().expect("should allocate");
            indices.push(idx);
        }
        
        assert_eq!(allocator.available(), 0);
        assert!(!allocator.is_empty());
        assert!(allocator.is_full());
        
        // Should fail when full
        assert!(allocator.allocate().is_none());
        
        // Free all slots
        for idx in indices {
            allocator.free(idx);
        }
        
        assert_eq!(allocator.available(), capacity);
        assert!(allocator.is_empty());
        assert!(!allocator.is_full());
    }
    
    #[test]
    fn test_stack_allocator() {
        test_allocator(StackAllocator::new(100));
    }
    
    #[test]
    fn test_freelist_allocator() {
        test_allocator(FreeListAllocator::new(100));
    }
    
    #[test]
    fn test_bitmap_allocator() {
        test_allocator(BitmapAllocator::new(100));
    }
}
