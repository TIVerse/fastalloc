//! Growth strategies for dynamic memory pools.

use alloc::boxed::Box;

/// Strategy for growing a memory pool when it runs out of capacity.
///
/// # Examples
///
/// ```rust
/// use fastalloc::GrowthStrategy;
///
/// // Don't grow (fixed-size pool)
/// let strategy = GrowthStrategy::None;
///
/// // Grow by a fixed amount
/// let strategy = GrowthStrategy::Linear { amount: 100 };
///
/// // Grow exponentially (double each time)
/// let strategy = GrowthStrategy::Exponential { factor: 2.0 };
///
/// // Custom growth logic
/// let strategy = GrowthStrategy::Custom {
///     compute: Box::new(|current| current + 50),
/// };
/// ```
pub enum GrowthStrategy {
    /// No growth - pool has fixed capacity.
    None,

    /// Grow by a fixed amount each time.
    Linear {
        /// Number of elements to add on each growth
        amount: usize,
    },

    /// Grow exponentially by multiplying current capacity.
    Exponential {
        /// Factor to multiply capacity by (e.g., 2.0 to double)
        factor: f64,
    },

    /// Custom growth function.
    ///
    /// The function receives the current capacity and returns the amount to grow by.
    Custom {
        /// Function that computes growth amount from current capacity
        compute: Box<dyn Fn(usize) -> usize + Send + Sync>,
    },
}

impl GrowthStrategy {
    /// Computes the growth amount for a given current capacity.
    pub fn compute_growth(&self, current_capacity: usize) -> usize {
        match self {
            GrowthStrategy::None => 0,
            GrowthStrategy::Linear { amount } => *amount,
            GrowthStrategy::Exponential { factor } => {
                let growth = (current_capacity as f64 * factor) as usize;
                growth.saturating_sub(current_capacity).max(1)
            }
            GrowthStrategy::Custom { compute } => compute(current_capacity),
        }
    }

    /// Returns whether this strategy allows growth.
    #[inline]
    pub fn allows_growth(&self) -> bool {
        !matches!(self, GrowthStrategy::None)
    }
}

impl core::fmt::Debug for GrowthStrategy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GrowthStrategy::None => write!(f, "GrowthStrategy::None"),
            GrowthStrategy::Linear { amount } => f
                .debug_struct("GrowthStrategy::Linear")
                .field("amount", amount)
                .finish(),
            GrowthStrategy::Exponential { factor } => f
                .debug_struct("GrowthStrategy::Exponential")
                .field("factor", factor)
                .finish(),
            GrowthStrategy::Custom { .. } => {
                write!(f, "GrowthStrategy::Custom {{ .. }}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_strategy_none() {
        let strategy = GrowthStrategy::None;
        assert_eq!(strategy.compute_growth(100), 0);
        assert!(!strategy.allows_growth());
    }

    #[test]
    fn growth_strategy_linear() {
        let strategy = GrowthStrategy::Linear { amount: 50 };
        assert_eq!(strategy.compute_growth(100), 50);
        assert_eq!(strategy.compute_growth(0), 50);
        assert!(strategy.allows_growth());
    }

    #[test]
    fn growth_strategy_exponential() {
        let strategy = GrowthStrategy::Exponential { factor: 2.0 };
        assert_eq!(strategy.compute_growth(100), 100); // 200 - 100
        assert_eq!(strategy.compute_growth(50), 50); // 100 - 50
        assert!(strategy.allows_growth());
    }

    #[test]
    fn growth_strategy_custom() {
        let strategy = GrowthStrategy::Custom {
            compute: Box::new(|current| current / 2),
        };
        assert_eq!(strategy.compute_growth(100), 50);
        assert_eq!(strategy.compute_growth(200), 100);
        assert!(strategy.allows_growth());
    }

    #[test]
    fn growth_strategy_exponential_minimum() {
        let strategy = GrowthStrategy::Exponential { factor: 2.0 };
        // Even with capacity 0, should grow by at least 1
        assert_eq!(strategy.compute_growth(0), 1);
    }
}
