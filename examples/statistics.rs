//! Statistics collection and monitoring example.

#[cfg(feature = "stats")]
fn main() {
    use fastalloc::{FixedPool, GrowingPool, GrowthStrategy, PoolConfig};
    use fastalloc::stats::StatisticsReporter;
    
    println!("=== Pool Statistics Example ===\n");
    
    // Example 1: Fixed pool statistics
    println!("1. Fixed Pool Statistics:");
    let pool = FixedPool::<i32>::new(100).expect("Failed to create pool");
    
    let mut handles = Vec::new();
    
    // Allocate some objects
    for i in 0..50 {
        handles.push(pool.allocate(i).expect("Allocation failed"));
    }
    
    println!("   Allocated 50 objects");
    
    let stats = pool.statistics();
    println!("\n{}", stats);
    
    // Free some objects
    handles.drain(0..20);
    
    println!("   Freed 20 objects\n");
    
    let stats = pool.statistics();
    let reporter = StatisticsReporter::new(stats);
    println!("{}", reporter.compact_summary());
    
    // Example 2: Growing pool statistics
    println!("\n2. Growing Pool Statistics:");
    
    let config = PoolConfig::builder()
        .capacity(10)
        .max_capacity(Some(100))
        .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
        .build()
        .unwrap();
    
    let growing_pool = GrowingPool::<String>::with_config(config).unwrap();
    
    let mut strings = Vec::new();
    
    // Trigger multiple growths
    for i in 0..50 {
        strings.push(growing_pool.allocate(format!("String-{}", i)).unwrap());
    }
    
    let stats = growing_pool.statistics();
    println!("\n{}", stats);
    
    println!("   Growth count: {}", stats.growth_count);
    println!("   Final capacity: {}", stats.capacity);
    
    // Example 3: Monitoring allocation patterns
    println!("\n3. Allocation Pattern Analysis:");
    
    let monitor_pool = FixedPool::<u64>::new(50).expect("Failed to create pool");
    
    // Simulate bursty allocation pattern
    for burst in 0..3 {
        println!("\n   Burst {}:", burst + 1);
        
        let mut burst_handles = Vec::new();
        for i in 0..15 {
            burst_handles.push(monitor_pool.allocate((burst * 100 + i) as u64).unwrap());
        }
        
        let stats = monitor_pool.statistics();
        println!("     Allocated: {}/{}", stats.current_usage, stats.capacity);
        println!("     Utilization: {:.1}%", stats.utilization_rate());
        
        // Release half
        burst_handles.drain(0..7);
    }
    
    let final_stats = monitor_pool.statistics();
    println!("\n   Final Statistics:");
    println!("     Total allocations: {}", final_stats.total_allocations);
    println!("     Total deallocations: {}", final_stats.total_deallocations);
    println!("     Peak usage: {}/{} ({:.1}%)", 
             final_stats.peak_usage,
             final_stats.capacity,
             final_stats.peak_utilization_rate());
    
    // Example 4: Key-value pairs for monitoring systems
    println!("\n4. Metrics for Monitoring Systems:");
    
    let stats = monitor_pool.statistics();
    let reporter = StatisticsReporter::new(stats);
    
    println!("\n   Key-Value Pairs (suitable for Prometheus, etc.):");
    for (key, value) in reporter.as_key_value_pairs() {
        println!("   pool.{} = {}", key, value);
    }
    
    // Example 5: Resetting statistics
    println!("\n5. Statistics Reset:");
    
    println!("   Before reset:");
    let stats = monitor_pool.statistics();
    println!("     Total allocations: {}", stats.total_allocations);
    
    monitor_pool.reset_statistics();
    
    println!("   After reset:");
    let stats = monitor_pool.statistics();
    println!("     Total allocations: {}", stats.total_allocations);
    println!("     Current usage: {} (preserved)", stats.current_usage);
    
    println!("\n=== Example Complete ===");
}

#[cfg(not(feature = "stats"))]
fn main() {
    println!("This example requires the 'stats' feature.");
    println!("Run with: cargo run --example statistics --features stats");
}
