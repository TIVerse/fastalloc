//! Statistics collection and reporting for memory pools.

mod collector;
mod reporter;

pub use collector::StatisticsCollector;
pub use reporter::StatisticsReporter;

use core::fmt;

/// Statistics about pool usage and performance.
///
/// # Examples
///
/// ```rust
/// #[cfg(feature = "stats")]
/// {
///     use fastalloc::FixedPool;
///
///     let pool = FixedPool::<i32>::new(100).unwrap();
///     
///     let _h1 = pool.allocate(1).unwrap();
///     let _h2 = pool.allocate(2).unwrap();
///     
///     let stats = pool.statistics();
///     assert_eq!(stats.current_usage, 2);
///     assert_eq!(stats.total_allocations, 2);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolStatistics {
    /// Total number of allocations ever made
    pub total_allocations: usize,

    /// Total number of deallocations ever made
    pub total_deallocations: usize,

    /// Current number of allocated objects
    pub current_usage: usize,

    /// Peak number of simultaneously allocated objects
    pub peak_usage: usize,

    /// Current capacity of the pool
    pub capacity: usize,

    /// Number of times the pool has grown (for growing pools)
    pub growth_count: usize,

    /// Number of allocation failures
    pub allocation_failures: usize,
}

impl PoolStatistics {
    /// Creates a new statistics instance with all counters at zero.
    pub fn new(capacity: usize) -> Self {
        Self {
            total_allocations: 0,
            total_deallocations: 0,
            current_usage: 0,
            peak_usage: 0,
            capacity,
            growth_count: 0,
            allocation_failures: 0,
        }
    }

    /// Returns the utilization rate as a percentage (0.0 to 100.0).
    #[inline]
    pub fn utilization_rate(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.current_usage as f64 / self.capacity as f64) * 100.0
        }
    }

    /// Returns the peak utilization rate as a percentage (0.0 to 100.0).
    #[inline]
    pub fn peak_utilization_rate(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.peak_usage as f64 / self.capacity as f64) * 100.0
        }
    }

    /// Returns the hit rate (successful allocations / total attempts).
    #[inline]
    pub fn hit_rate(&self) -> f64 {
        let total_attempts = self.total_allocations + self.allocation_failures;
        if total_attempts == 0 {
            1.0
        } else {
            self.total_allocations as f64 / total_attempts as f64
        }
    }

    /// Returns the number of currently available slots.
    #[inline]
    pub fn available(&self) -> usize {
        self.capacity.saturating_sub(self.current_usage)
    }
}

impl fmt::Display for PoolStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Pool Statistics:")?;
        writeln!(f, "  Capacity:            {}", self.capacity)?;
        writeln!(
            f,
            "  Current Usage:       {} ({:.1}%)",
            self.current_usage,
            self.utilization_rate()
        )?;
        writeln!(
            f,
            "  Peak Usage:          {} ({:.1}%)",
            self.peak_usage,
            self.peak_utilization_rate()
        )?;
        writeln!(f, "  Total Allocations:   {}", self.total_allocations)?;
        writeln!(f, "  Total Deallocations: {}", self.total_deallocations)?;
        writeln!(f, "  Allocation Failures: {}", self.allocation_failures)?;
        writeln!(f, "  Hit Rate:            {:.2}%", self.hit_rate() * 100.0)?;
        writeln!(f, "  Growth Count:        {}", self.growth_count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_utilization() {
        let stats = PoolStatistics {
            capacity: 100,
            current_usage: 50,
            peak_usage: 75,
            ..PoolStatistics::new(100)
        };

        assert_eq!(stats.utilization_rate(), 50.0);
        assert_eq!(stats.peak_utilization_rate(), 75.0);
    }

    #[test]
    fn statistics_hit_rate() {
        let stats = PoolStatistics {
            total_allocations: 90,
            allocation_failures: 10,
            ..PoolStatistics::new(100)
        };

        assert_eq!(stats.hit_rate(), 0.9);
    }

    #[test]
    fn statistics_available() {
        let stats = PoolStatistics {
            capacity: 100,
            current_usage: 30,
            ..PoolStatistics::new(100)
        };

        assert_eq!(stats.available(), 70);
    }
}
