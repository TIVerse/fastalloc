//! Free-list allocator implementation.

use super::Allocator;
use alloc::vec::Vec;

/// A free-list allocator that maintains a list of available slots.
///
/// This allocator provides O(1) allocation and deallocation while
/// supporting efficient random access patterns.
///
/// Time complexity: O(1) for both allocation and deallocation.
/// Space complexity: O(capacity) for storing free indices.
pub(crate) struct FreeListAllocator {
    /// List of free indices
    free_list: Vec<usize>,
    /// Total capacity
    capacity: usize,
}

impl FreeListAllocator {
    /// Creates a new free-list allocator with the given capacity.
    pub fn new(capacity: usize) -> Self {
        // Initialize with all indices available
        let free_list: Vec<usize> = (0..capacity).collect();
        
        Self {
            free_list,
            capacity,
        }
    }
    
    /// Extends the allocator with additional capacity.
    pub fn extend(&mut self, additional: usize) {
        let old_capacity = self.capacity;
        self.capacity += additional;
        
        // Add new indices to the free list
        self.free_list.extend(old_capacity..self.capacity);
    }
}

impl Allocator for FreeListAllocator {
    #[inline]
    fn allocate(&mut self) -> Option<usize> {
        self.free_list.pop()
    }
    
    #[inline]
    fn free(&mut self, index: usize) {
        debug_assert!(index < self.capacity, "index out of bounds");
        debug_assert!(
            !self.free_list.contains(&index),
            "double free detected"
        );
        self.free_list.push(index);
    }
    
    #[inline]
    fn available(&self) -> usize {
        self.free_list.len()
    }
    
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new_allocator_is_empty() {
        let allocator = FreeListAllocator::new(10);
        assert_eq!(allocator.available(), 10);
        assert_eq!(allocator.capacity(), 10);
        assert!(allocator.is_empty());
    }
    
    #[test]
    fn allocate_and_free() {
        let mut allocator = FreeListAllocator::new(5);
        
        let idx0 = allocator.allocate().unwrap();
        let idx1 = allocator.allocate().unwrap();
        assert_eq!(allocator.available(), 3);
        
        allocator.free(idx0);
        assert_eq!(allocator.available(), 4);
        
        allocator.free(idx1);
        assert_eq!(allocator.available(), 5);
        assert!(allocator.is_empty());
    }
    
    #[test]
    fn allocate_until_full() {
        let mut allocator = FreeListAllocator::new(3);
        
        assert!(allocator.allocate().is_some());
        assert!(allocator.allocate().is_some());
        assert!(allocator.allocate().is_some());
        assert!(allocator.is_full());
        assert!(allocator.allocate().is_none());
    }
    
    #[test]
    fn extend_capacity() {
        let mut allocator = FreeListAllocator::new(2);
        
        allocator.allocate();
        allocator.allocate();
        assert!(allocator.is_full());
        
        allocator.extend(3);
        assert_eq!(allocator.capacity(), 5);
        assert_eq!(allocator.available(), 3);
        assert!(!allocator.is_full());
        
        assert!(allocator.allocate().is_some());
        assert!(allocator.allocate().is_some());
        assert!(allocator.allocate().is_some());
        assert!(allocator.is_full());
    }
    
    #[test]
    fn reuse_freed_slots() {
        let mut allocator = FreeListAllocator::new(3);
        
        let idx0 = allocator.allocate().unwrap();
        let _idx1 = allocator.allocate().unwrap();
        
        allocator.free(idx0);
        
        let idx2 = allocator.allocate().unwrap();
        // Should reuse the freed slot
        assert_eq!(idx2, idx0);
    }
}
