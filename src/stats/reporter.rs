//! Statistics reporting utilities.

use super::PoolStatistics;
use alloc::string::String;
use alloc::vec::Vec;

/// Formats and reports pool statistics in various formats.
///
/// # Examples
///
/// ```rust
/// #[cfg(feature = "stats")]
/// {
///     use fastalloc::stats::{PoolStatistics, StatisticsReporter};
///
///     let stats = PoolStatistics::new(100);
///     let reporter = StatisticsReporter::new(stats);
///     
///     let summary = reporter.summary();
///     assert!(summary.contains("Capacity: 100"));
/// }
/// ```
pub struct StatisticsReporter {
    stats: PoolStatistics,
}

impl StatisticsReporter {
    /// Creates a new reporter for the given statistics.
    pub fn new(stats: PoolStatistics) -> Self {
        Self { stats }
    }
    
    /// Returns a human-readable summary of the statistics.
    pub fn summary(&self) -> String {
        alloc::format!("{}", self.stats)
    }
    
    /// Returns a compact one-line summary.
    pub fn compact_summary(&self) -> String {
        alloc::format!(
            "Pool: {}/{} ({:.1}%) | Allocs: {} | Deallocs: {} | Failures: {}",
            self.stats.current_usage,
            self.stats.capacity,
            self.stats.utilization_rate(),
            self.stats.total_allocations,
            self.stats.total_deallocations,
            self.stats.allocation_failures
        )
    }
    
    /// Returns statistics as key-value pairs.
    pub fn as_key_value_pairs(&self) -> Vec<(&'static str, String)> {
        alloc::vec![
            ("capacity", self.stats.capacity.to_string()),
            ("current_usage", self.stats.current_usage.to_string()),
            ("peak_usage", self.stats.peak_usage.to_string()),
            ("utilization_rate", alloc::format!("{:.2}%", self.stats.utilization_rate())),
            ("total_allocations", self.stats.total_allocations.to_string()),
            ("total_deallocations", self.stats.total_deallocations.to_string()),
            ("allocation_failures", self.stats.allocation_failures.to_string()),
            ("hit_rate", alloc::format!("{:.4}", self.stats.hit_rate())),
            ("growth_count", self.stats.growth_count.to_string()),
        ]
    }
    
    /// Returns statistics in JSON format (requires alloc).
    #[cfg(feature = "serde")]
    pub fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.stats)
    }
    
    /// Logs statistics using the tracing framework (requires tracing feature).
    #[cfg(feature = "tracing")]
    pub fn log(&self) {
        tracing::info!(
            capacity = self.stats.capacity,
            current_usage = self.stats.current_usage,
            peak_usage = self.stats.peak_usage,
            total_allocations = self.stats.total_allocations,
            total_deallocations = self.stats.total_deallocations,
            allocation_failures = self.stats.allocation_failures,
            growth_count = self.stats.growth_count,
            utilization_rate = %format!("{:.2}%", self.stats.utilization_rate()),
            "Pool statistics"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn reporter_summary() {
        let stats = PoolStatistics {
            capacity: 100,
            current_usage: 50,
            total_allocations: 100,
            ..PoolStatistics::new(100)
        };
        
        let reporter = StatisticsReporter::new(stats);
        let summary = reporter.summary();
        
        assert!(summary.contains("Capacity:"));
        assert!(summary.contains("100"));
    }
    
    #[test]
    fn reporter_compact_summary() {
        let stats = PoolStatistics {
            capacity: 100,
            current_usage: 25,
            total_allocations: 50,
            total_deallocations: 25,
            ..PoolStatistics::new(100)
        };
        
        let reporter = StatisticsReporter::new(stats);
        let compact = reporter.compact_summary();
        
        assert!(compact.contains("25/100"));
        assert!(compact.contains("Allocs: 50"));
    }
    
    #[test]
    fn reporter_key_value_pairs() {
        let stats = PoolStatistics::new(100);
        let reporter = StatisticsReporter::new(stats);
        
        let pairs = reporter.as_key_value_pairs();
        assert!(!pairs.is_empty());
        
        // Check that capacity is present
        let capacity_pair = pairs.iter().find(|(k, _)| *k == "capacity");
        assert!(capacity_pair.is_some());
    }
}
