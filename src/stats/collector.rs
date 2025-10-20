//! Statistics collector for tracking pool metrics.

use super::PoolStatistics;

/// Collects statistics about pool operations.
///
/// This is used internally by pool implementations to track metrics
/// when the `stats` feature is enabled.
pub struct StatisticsCollector {
    stats: PoolStatistics,
}

impl StatisticsCollector {
    /// Creates a new statistics collector.
    pub fn new(capacity: usize) -> Self {
        Self {
            stats: PoolStatistics::new(capacity),
        }
    }
    
    /// Records an allocation.
    #[inline]
    pub fn record_allocation(&mut self) {
        self.stats.total_allocations += 1;
        self.stats.current_usage += 1;
        
        if self.stats.current_usage > self.stats.peak_usage {
            self.stats.peak_usage = self.stats.current_usage;
        }
    }
    
    /// Records a deallocation.
    #[inline]
    pub fn record_deallocation(&mut self) {
        self.stats.total_deallocations += 1;
        self.stats.current_usage = self.stats.current_usage.saturating_sub(1);
    }
    
    /// Records an allocation failure.
    #[inline]
    pub fn record_failure(&mut self) {
        self.stats.allocation_failures += 1;
    }
    
    /// Records pool growth.
    #[inline]
    pub fn record_growth(&mut self, new_capacity: usize) {
        self.stats.growth_count += 1;
        self.stats.capacity = new_capacity;
    }
    
    /// Returns a snapshot of the current statistics.
    #[inline]
    pub fn snapshot(&self) -> PoolStatistics {
        self.stats
    }
    
    /// Resets all statistics counters.
    pub fn reset(&mut self) {
        let capacity = self.stats.capacity;
        self.stats = PoolStatistics::new(capacity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn collector_tracks_allocations() {
        let mut collector = StatisticsCollector::new(100);
        
        collector.record_allocation();
        collector.record_allocation();
        collector.record_allocation();
        
        let stats = collector.snapshot();
        assert_eq!(stats.total_allocations, 3);
        assert_eq!(stats.current_usage, 3);
        assert_eq!(stats.peak_usage, 3);
    }
    
    #[test]
    fn collector_tracks_deallocations() {
        let mut collector = StatisticsCollector::new(100);
        
        collector.record_allocation();
        collector.record_allocation();
        collector.record_deallocation();
        
        let stats = collector.snapshot();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_deallocations, 1);
        assert_eq!(stats.current_usage, 1);
        assert_eq!(stats.peak_usage, 2);
    }
    
    #[test]
    fn collector_tracks_failures() {
        let mut collector = StatisticsCollector::new(100);
        
        collector.record_failure();
        collector.record_failure();
        
        let stats = collector.snapshot();
        assert_eq!(stats.allocation_failures, 2);
    }
    
    #[test]
    fn collector_tracks_growth() {
        let mut collector = StatisticsCollector::new(100);
        
        collector.record_growth(200);
        collector.record_growth(400);
        
        let stats = collector.snapshot();
        assert_eq!(stats.growth_count, 2);
        assert_eq!(stats.capacity, 400);
    }
    
    #[test]
    fn collector_reset() {
        let mut collector = StatisticsCollector::new(100);
        
        collector.record_allocation();
        collector.record_allocation();
        collector.record_failure();
        
        collector.reset();
        
        let stats = collector.snapshot();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.allocation_failures, 0);
        assert_eq!(stats.capacity, 100); // Capacity is preserved
    }
}
