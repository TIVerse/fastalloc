//! Stack-based (LIFO) allocator implementation.

use super::Allocator;
use alloc::vec::Vec;

/// A stack-based allocator that uses LIFO (Last-In-First-Out) allocation.
///
/// This allocator provides excellent cache locality since recently freed
/// objects are likely to still be in cache when reallocated.
///
/// Time complexity: O(1) for both allocation and deallocation.
/// Space complexity: O(capacity) for storing free indices.
pub(crate) struct StackAllocator {
    /// Stack of available indices (LIFO)
    free_stack: Vec<usize>,
    /// Total capacity
    capacity: usize,
    /// Debug-mode tracking for double-free detection
    #[cfg(debug_assertions)]
    allocated_bitmap: alloc::vec::Vec<u64>,
}

impl StackAllocator {
    /// Creates a new stack allocator with the given capacity.
    pub fn new(capacity: usize) -> Self {
        // Initialize with all indices available in reverse order
        // so that index 0 is allocated first
        let free_stack: Vec<usize> = (0..capacity).rev().collect();
        
        Self {
            free_stack,
            capacity,
            #[cfg(debug_assertions)]
            allocated_bitmap: {
                let num_words = (capacity + 63) / 64;
                alloc::vec![0u64; num_words]
            },
        }
    }
    
    /// Creates a new stack allocator with additional capacity.
    pub fn with_additional_capacity(&mut self, additional: usize) {
        let old_capacity = self.capacity;
        self.capacity += additional;
        
        #[cfg(debug_assertions)]
        {
            let new_num_words = (self.capacity + 63) / 64;
            self.allocated_bitmap.resize(new_num_words, 0);
        }
        
        // Add new indices to the stack
        for i in (old_capacity..self.capacity).rev() {
            self.free_stack.push(i);
        }
    }
}

impl Allocator for StackAllocator {
    #[inline]
    fn allocate(&mut self) -> Option<usize> {
        let index = self.free_stack.pop()?;
        
        #[cfg(debug_assertions)]
        {
            let word_idx = index / 64;
            let bit_pos = index % 64;
            debug_assert_eq!(
                self.allocated_bitmap[word_idx] & (1u64 << bit_pos),
                0,
                "allocating already allocated index {}",
                index
            );
            self.allocated_bitmap[word_idx] |= 1u64 << bit_pos;
        }
        
        Some(index)
    }
    
    #[inline]
    fn free(&mut self, index: usize) {
        debug_assert!(index < self.capacity, "index out of bounds");
        
        #[cfg(debug_assertions)]
        {
            let word_idx = index / 64;
            let bit_pos = index % 64;
            debug_assert_ne!(
                self.allocated_bitmap[word_idx] & (1u64 << bit_pos),
                0,
                "double free detected for index {}",
                index
            );
            self.allocated_bitmap[word_idx] &= !(1u64 << bit_pos);
        }
        
        self.free_stack.push(index);
    }
    
    #[inline]
    fn available(&self) -> usize {
        self.free_stack.len()
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
    fn new_allocator_has_all_slots_available() {
        let allocator = StackAllocator::new(10);
        assert_eq!(allocator.available(), 10);
        assert_eq!(allocator.capacity(), 10);
        assert!(allocator.is_empty());
        assert!(!allocator.is_full());
    }
    
    #[test]
    fn allocate_returns_indices_in_order() {
        let mut allocator = StackAllocator::new(5);
        
        // Should allocate 0, 1, 2, 3, 4 in order
        assert_eq!(allocator.allocate(), Some(0));
        assert_eq!(allocator.allocate(), Some(1));
        assert_eq!(allocator.allocate(), Some(2));
        assert_eq!(allocator.allocate(), Some(3));
        assert_eq!(allocator.allocate(), Some(4));
        assert_eq!(allocator.allocate(), None);
    }
    
    #[test]
    fn lifo_behavior() {
        let mut allocator = StackAllocator::new(3);
        
        let idx0 = allocator.allocate().unwrap();
        let idx1 = allocator.allocate().unwrap();
        let idx2 = allocator.allocate().unwrap();
        
        // Free in order: idx0, idx1, idx2
        allocator.free(idx0);
        allocator.free(idx1);
        allocator.free(idx2);
        
        // Should get them back in LIFO order: idx2, idx1, idx0
        assert_eq!(allocator.allocate(), Some(idx2));
        assert_eq!(allocator.allocate(), Some(idx1));
        assert_eq!(allocator.allocate(), Some(idx0));
    }
    
    #[test]
    fn with_additional_capacity() {
        let mut allocator = StackAllocator::new(2);
        
        allocator.allocate();
        allocator.allocate();
        assert!(allocator.is_full());
        
        allocator.with_additional_capacity(3);
        assert_eq!(allocator.capacity(), 5);
        assert_eq!(allocator.available(), 3);
        
        // Should be able to allocate new indices
        assert_eq!(allocator.allocate(), Some(2));
        assert_eq!(allocator.allocate(), Some(3));
        assert_eq!(allocator.allocate(), Some(4));
        assert!(allocator.is_full());
    }
}
