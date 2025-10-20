//! Bitmap-based allocator implementation.

use super::Allocator;
use alloc::vec::Vec;

/// A bitmap allocator that uses a bit vector to track free slots.
///
/// This allocator has excellent space efficiency, using only 1 bit per slot,
/// and provides good cache locality for the allocation metadata.
///
/// Time complexity: O(n) worst case for allocation (scanning for free bit),
/// but typically O(1) in practice due to hint tracking.
/// Space complexity: O(capacity/8) bits.
pub(crate) struct BitmapAllocator {
    /// Bitmap where each bit represents whether a slot is allocated (1) or free (0)
    bitmap: Vec<u64>,
    /// Total capacity (number of slots)
    capacity: usize,
    /// Number of allocated slots
    allocated: usize,
    /// Hint for next free word to check (optimization)
    next_free_hint: usize,
}

impl BitmapAllocator {
    const BITS_PER_WORD: usize = 64;
    
    /// Creates a new bitmap allocator with the given capacity.
    pub fn new(capacity: usize) -> Self {
        let num_words = (capacity + Self::BITS_PER_WORD - 1) / Self::BITS_PER_WORD;
        let bitmap = alloc::vec![0u64; num_words];
        
        Self {
            bitmap,
            capacity,
            allocated: 0,
            next_free_hint: 0,
        }
    }
    
    /// Returns the word index and bit position for a given slot index.
    #[inline]
    fn word_and_bit(index: usize) -> (usize, usize) {
        let word_idx = index / Self::BITS_PER_WORD;
        let bit_pos = index % Self::BITS_PER_WORD;
        (word_idx, bit_pos)
    }
    
    /// Checks if a slot is allocated.
    #[inline]
    fn is_allocated(&self, index: usize) -> bool {
        let (word_idx, bit_pos) = Self::word_and_bit(index);
        (self.bitmap[word_idx] & (1u64 << bit_pos)) != 0
    }
    
    /// Marks a slot as allocated.
    #[inline]
    fn mark_allocated(&mut self, index: usize) {
        let (word_idx, bit_pos) = Self::word_and_bit(index);
        self.bitmap[word_idx] |= 1u64 << bit_pos;
    }
    
    /// Marks a slot as free.
    #[inline]
    fn mark_free(&mut self, index: usize) {
        let (word_idx, bit_pos) = Self::word_and_bit(index);
        self.bitmap[word_idx] &= !(1u64 << bit_pos);
    }
    
    /// Finds the next free slot starting from the hint.
    #[inline]
    fn find_free_slot(&mut self) -> Option<usize> {
        let num_words = self.bitmap.len();
        
        // Search starting from hint
        for offset in 0..num_words {
            let word_idx = (self.next_free_hint + offset) % num_words;
            let word = self.bitmap[word_idx];
            
            // If word is not all ones, there's a free bit
            if word != u64::MAX {
                // Find first zero bit using fast CPU instruction
                let bit_pos = (!word).trailing_zeros() as usize;
                let index = word_idx * Self::BITS_PER_WORD + bit_pos;
                
                // Make sure the index is within capacity
                if index < self.capacity {
                    self.next_free_hint = word_idx;
                    return Some(index);
                }
            }
        }
        
        None
    }
    
    /// Attempts to find multiple free slots at once for batch operations.
    ///
    /// This is more efficient than calling find_free_slot multiple times.
    pub fn find_free_slots(&mut self, count: usize) -> Option<alloc::vec::Vec<usize>> {
        if count > self.available() {
            return None;
        }
        
        let mut indices = alloc::vec::Vec::with_capacity(count);
        
        for _ in 0..count {
            if let Some(index) = self.find_free_slot() {
                indices.push(index);
            } else {
                // Shouldn't happen given the check above
                return None;
            }
        }
        
        Some(indices)
    }
    
    /// Extends the allocator with additional capacity.
    pub fn extend(&mut self, additional: usize) {
        self.capacity += additional;
        
        let new_num_words = (self.capacity + Self::BITS_PER_WORD - 1) / Self::BITS_PER_WORD;
        let old_num_words = self.bitmap.len();
        
        // Add more words if needed
        if new_num_words > old_num_words {
            self.bitmap.resize(new_num_words, 0u64);
        }
    }
}

impl Allocator for BitmapAllocator {
    fn allocate(&mut self) -> Option<usize> {
        if self.allocated >= self.capacity {
            return None;
        }
        
        let index = self.find_free_slot()?;
        self.mark_allocated(index);
        self.allocated += 1;
        
        Some(index)
    }
    
    fn free(&mut self, index: usize) {
        debug_assert!(index < self.capacity, "index out of bounds");
        debug_assert!(self.is_allocated(index), "double free detected");
        
        self.mark_free(index);
        self.allocated -= 1;
        
        // Update hint to this word for faster subsequent allocations
        let (word_idx, _) = Self::word_and_bit(index);
        self.next_free_hint = word_idx;
    }
    
    #[inline]
    fn available(&self) -> usize {
        self.capacity - self.allocated
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
        let allocator = BitmapAllocator::new(100);
        assert_eq!(allocator.available(), 100);
        assert_eq!(allocator.capacity(), 100);
        assert!(allocator.is_empty());
    }
    
    #[test]
    fn allocate_and_free() {
        let mut allocator = BitmapAllocator::new(10);
        
        let idx0 = allocator.allocate().unwrap();
        let idx1 = allocator.allocate().unwrap();
        assert_eq!(allocator.available(), 8);
        assert_eq!(allocator.allocated, 2);
        
        allocator.free(idx0);
        assert_eq!(allocator.available(), 9);
        assert_eq!(allocator.allocated, 1);
        
        allocator.free(idx1);
        assert!(allocator.is_empty());
    }
    
    #[test]
    fn allocate_until_full() {
        let mut allocator = BitmapAllocator::new(5);
        
        for _ in 0..5 {
            assert!(allocator.allocate().is_some());
        }
        
        assert!(allocator.is_full());
        assert!(allocator.allocate().is_none());
    }
    
    #[test]
    fn extend_capacity() {
        let mut allocator = BitmapAllocator::new(2);
        
        allocator.allocate();
        allocator.allocate();
        assert!(allocator.is_full());
        
        allocator.extend(3);
        assert_eq!(allocator.capacity(), 5);
        assert_eq!(allocator.available(), 3);
        
        assert!(allocator.allocate().is_some());
        assert!(allocator.allocate().is_some());
        assert!(allocator.allocate().is_some());
        assert!(allocator.is_full());
    }
    
    #[test]
    fn large_capacity() {
        let mut allocator = BitmapAllocator::new(1000);
        
        // Allocate many slots
        let mut indices = Vec::new();
        for _ in 0..500 {
            indices.push(allocator.allocate().unwrap());
        }
        
        assert_eq!(allocator.available(), 500);
        
        // Free them
        for idx in indices {
            allocator.free(idx);
        }
        
        assert!(allocator.is_empty());
    }
    
    #[test]
    fn reuse_freed_slots() {
        let mut allocator = BitmapAllocator::new(10);
        
        let idx = allocator.allocate().unwrap();
        allocator.free(idx);
        
        let reused_idx = allocator.allocate().unwrap();
        assert_eq!(reused_idx, idx);
    }
}
