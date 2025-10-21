//! Utility functions for alignment and size calculations.

use crate::error::{Error, Result};

/// Checks if a value is a power of two.
#[inline]
pub const fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Validates that an alignment value is valid (must be a power of two).
#[inline]
pub fn validate_alignment(alignment: usize) -> Result<()> {
    if is_power_of_two(alignment) {
        Ok(())
    } else {
        Err(Error::InvalidAlignment { alignment })
    }
}

/// Rounds up a size to the next multiple of alignment.
#[inline]
#[allow(dead_code)]
pub const fn align_up(size: usize, alignment: usize) -> usize {
    (size + alignment - 1) & !(alignment - 1)
}

/// Calculates the aligned size for a type with custom alignment.
#[inline]
#[allow(dead_code)]
pub const fn aligned_size<T>(alignment: usize) -> usize {
    align_up(core::mem::size_of::<T>(), alignment)
}

/// Calculates padding needed to reach alignment.
#[inline]
#[allow(dead_code)]
pub const fn padding_needed(size: usize, alignment: usize) -> usize {
    let aligned = align_up(size, alignment);
    aligned - size
}

/// Computes the growth amount based on current capacity and growth factor.
#[inline]
pub fn compute_exponential_growth(current: usize, factor: f64) -> usize {
    let growth = (current as f64 * factor) as usize;
    growth.max(1) // Ensure at least 1 element of growth
}

/// Computes linear growth.
#[inline]
#[allow(dead_code)]
pub const fn compute_linear_growth(amount: usize) -> usize {
    amount
}

/// Ensures a capacity value is within bounds.
#[inline]
#[allow(dead_code)]
pub fn clamp_capacity(value: usize, min: usize, max: Option<usize>) -> usize {
    let clamped = value.max(min);
    if let Some(max_val) = max {
        clamped.min(max_val)
    } else {
        clamped
    }
}

/// Computes the next chunk size for a growing pool.
#[allow(dead_code)]
pub fn next_chunk_size(
    current_capacity: usize,
    growth_strategy: &crate::config::GrowthStrategy,
) -> usize {
    match growth_strategy {
        crate::config::GrowthStrategy::None => 0,
        crate::config::GrowthStrategy::Linear { amount } => *amount,
        crate::config::GrowthStrategy::Exponential { factor } => {
            compute_exponential_growth(current_capacity, *factor)
        }
        crate::config::GrowthStrategy::Custom { compute } => compute(current_capacity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(8));
        assert!(is_power_of_two(64));
        assert!(is_power_of_two(1024));

        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(7));
        assert!(!is_power_of_two(100));
    }

    #[test]
    fn test_validate_alignment() {
        assert!(validate_alignment(1).is_ok());
        assert!(validate_alignment(2).is_ok());
        assert!(validate_alignment(4).is_ok());
        assert!(validate_alignment(64).is_ok());

        assert!(validate_alignment(0).is_err());
        assert!(validate_alignment(3).is_err());
        assert!(validate_alignment(7).is_err());
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
        assert_eq!(align_up(7, 8), 8);
        assert_eq!(align_up(9, 8), 16);
        assert_eq!(align_up(100, 64), 128);
    }

    #[test]
    fn test_padding_needed() {
        assert_eq!(padding_needed(0, 4), 0);
        assert_eq!(padding_needed(1, 4), 3);
        assert_eq!(padding_needed(4, 4), 0);
        assert_eq!(padding_needed(5, 4), 3);
        assert_eq!(padding_needed(7, 8), 1);
        assert_eq!(padding_needed(9, 8), 7);
    }

    #[test]
    fn test_compute_exponential_growth() {
        assert_eq!(compute_exponential_growth(100, 2.0), 200);
        assert_eq!(compute_exponential_growth(100, 1.5), 150);
        assert_eq!(compute_exponential_growth(0, 2.0), 1); // Minimum growth
    }

    #[test]
    fn test_clamp_capacity() {
        assert_eq!(clamp_capacity(50, 10, Some(100)), 50);
        assert_eq!(clamp_capacity(5, 10, Some(100)), 10);
        assert_eq!(clamp_capacity(150, 10, Some(100)), 100);
        assert_eq!(clamp_capacity(150, 10, None), 150);
    }
}
