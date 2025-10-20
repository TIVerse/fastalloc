use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fastalloc::{FixedPool, GrowingPool, GrowthStrategy, PoolConfig};
use std::collections::VecDeque;

fn bench_fragmentation_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmentation");
    
    // Alternating allocation/deallocation pattern
    group.bench_function("alternating_alloc_dealloc", |b| {
        let pool = FixedPool::<i32>::new(1000).unwrap();
        
        b.iter(|| {
            let mut handles = Vec::new();
            
            // Allocate every other slot
            for i in 0..500 {
                handles.push(pool.allocate(i).unwrap());
                if let Ok(h) = pool.allocate(i + 1000) {
                    drop(h); // Immediately deallocate
                }
            }
            
            // Clean up
            handles.clear();
        });
    });
    
    // FIFO pattern (queue-like)
    group.bench_function("fifo_pattern", |b| {
        let pool = FixedPool::<i32>::new(100).unwrap();
        
        b.iter(|| {
            let mut queue = VecDeque::new();
            
            // Fill queue
            for i in 0..100 {
                queue.push_back(pool.allocate(i).unwrap());
            }
            
            // Process in FIFO order
            for i in 100..200 {
                queue.pop_front(); // Deallocate oldest
                queue.push_back(pool.allocate(i).unwrap());
            }
            
            queue.clear();
        });
    });
    
    // Random access pattern
    group.bench_function("random_lifetime", |b| {
        let pool = FixedPool::<i32>::new(500).unwrap();
        
        b.iter(|| {
            let mut handles = Vec::new();
            
            for i in 0..500 {
                handles.push(pool.allocate(i).unwrap());
                
                // Randomly drop some handles
                if i % 3 == 0 && !handles.is_empty() {
                    let idx = i % handles.len();
                    handles.remove(idx);
                }
            }
            
            handles.clear();
        });
    });
    
    group.finish();
}

fn bench_growing_pool_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("growing_pool_fragmentation");
    
    group.bench_function("linear_growth", |b| {
        let config = PoolConfig::builder()
            .capacity(100)
            .max_capacity(Some(1000))
            .growth_strategy(GrowthStrategy::Linear { amount: 100 })
            .build()
            .unwrap();
        let pool = GrowingPool::with_config(config).unwrap();
        
        b.iter(|| {
            let mut handles = Vec::new();
            
            // Grow pool through multiple allocations
            for i in 0..500 {
                if let Ok(h) = pool.allocate(i) {
                    handles.push(h);
                }
                
                // Deallocate every 5th
                if i % 5 == 0 && !handles.is_empty() {
                    handles.remove(0);
                }
            }
            
            handles.clear();
        });
    });
    
    group.bench_function("exponential_growth", |b| {
        let config = PoolConfig::builder()
            .capacity(10)
            .max_capacity(Some(1000))
            .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
            .build()
            .unwrap();
        let pool = GrowingPool::with_config(config).unwrap();
        
        b.iter(|| {
            let mut handles = Vec::new();
            
            for i in 0..500 {
                if let Ok(h) = pool.allocate(i) {
                    handles.push(h);
                }
                
                if i % 7 == 0 && !handles.is_empty() {
                    handles.pop();
                }
            }
            
            handles.clear();
        });
    });
    
    group.finish();
}

fn bench_long_running(c: &mut Criterion) {
    let mut group = c.benchmark_group("long_running");
    group.sample_size(10); // Reduce samples for long-running benchmark
    
    group.bench_function("sustained_load", |b| {
        let pool = FixedPool::<i32>::new(1000).unwrap();
        
        b.iter(|| {
            let mut handles = Vec::new();
            
            // Simulate sustained workload
            for cycle in 0..100 {
                // Allocate batch
                for i in 0..100 {
                    if let Ok(h) = pool.allocate(cycle * 100 + i) {
                        handles.push(h);
                    }
                }
                
                // Deallocate older batch
                if handles.len() > 500 {
                    handles.drain(0..100);
                }
            }
            
            black_box(&handles);
            handles.clear();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_fragmentation_pattern,
    bench_growing_pool_fragmentation,
    bench_long_running
);
criterion_main!(benches);
