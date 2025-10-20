# Performance Tuning Guide

This guide helps you get the best performance from fastalloc.

## Choosing the Right Pool Type

### FixedPool
**Use when:**
- You know the maximum number of objects needed
- You want the absolute fastest allocation/deallocation
- You need predictable, zero-fragmentation behavior
- Memory constraints are known upfront

**Performance:**
- Allocation: 10-20ns
- Deallocation: 5-10ns
- Zero fragmentation
- Best cache locality

```rust
let pool = FixedPool::<Entity>::new(10000).unwrap();
```

### GrowingPool
**Use when:**
- Object count varies significantly
- You want to start small and grow on demand
- You need to balance memory usage vs. performance
- You have a reasonable maximum capacity

**Performance:**
- Allocation: 15-50ns (spikes during growth)
- Deallocation: 10-15ns
- Minimal fragmentation with good strategies
- Slightly reduced cache locality

```rust
let config = PoolConfig::builder()
    .capacity(100)
    .max_capacity(Some(10000))
    .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
    .build()
    .unwrap();
let pool = GrowingPool::with_config(config).unwrap();
```

### ThreadLocalPool
**Use when:**
- Each thread needs its own pool
- You want zero synchronization overhead
- Objects don't need to cross thread boundaries
- Single-threaded performance is critical

**Performance:**
- Same as FixedPool (10-20ns allocation)
- Zero lock contention
- Best for thread-affine workloads

```rust
let pool = ThreadLocalPool::<Data>::new(1000).unwrap();
```

### ThreadSafePool
**Use when:**
- Multiple threads need to share a pool
- Allocation rate per thread is moderate
- You need simple concurrent access

**Performance:**
- Allocation: 50-100ns (with moderate contention)
- Higher under contention
- Use parking_lot feature for 20-30% improvement

```rust
let pool = Arc::new(ThreadSafePool::<Item>::new(5000).unwrap());
```

## Capacity Planning

### Initial Capacity
Choose based on typical workload:

```rust
// Gaming: spawn waves
let pool = FixedPool::new(100).unwrap(); // per-wave capacity

// Server: concurrent connections
let pool = ThreadSafePool::new(1000).unwrap(); // expected load

// Data processing: batch size
let pool = FixedPool::new(batch_size * 2).unwrap(); // double-buffering
```

### Growth Strategy

**Linear:** Predictable memory growth
```rust
GrowthStrategy::Linear { amount: 100 }
// Good for: steady load increases
```

**Exponential:** Fast adaptation to demand
```rust
GrowthStrategy::Exponential { factor: 2.0 }
// Good for: bursty workloads, unknown peaks
```

**Custom:** Application-specific logic
```rust
GrowthStrategy::Custom {
    compute: Box::new(|current| {
        // Grow by 50%, minimum 10
        std::cmp::max(10, current / 2)
    })
}
```

## Memory Alignment

### Cache-Line Alignment
Prevent false sharing in concurrent scenarios:

```rust
PoolConfig::builder()
    .capacity(1000)
    .alignment(64) // Typical cache line size
    .build()
    .unwrap();
```

**When to use:**
- Thread-safe pools with high contention
- Objects accessed by multiple threads
- Performance-critical data structures

### SIMD Alignment
For vectorized operations:

```rust
PoolConfig::builder()
    .capacity(1000)
    .alignment(32) // AVX alignment
    .build()
    .unwrap();
```

## Initialization Strategies

### Lazy (Default)
Initialize objects when first allocated:

```rust
// Implicit - default behavior
let pool = FixedPool::new(1000).unwrap();
```

**Pros:**
- Fast pool creation
- No wasted initialization

**Cons:**
- First allocation slightly slower

### Eager
Pre-initialize all objects:

```rust
PoolConfig::builder()
    .capacity(1000)
    .pre_initialize(true)
    .initializer(|| MyType::default())
    .build()
    .unwrap();
```

**Pros:**
- Consistent allocation speed
- Front-load initialization cost

**Cons:**
- Slower pool creation
- Uses more memory upfront

### Custom with Reset
Reuse and reset objects:

```rust
PoolConfig::builder()
    .capacity(1000)
    .reset_fn(
        || Vec::with_capacity(1024),
        |v| {
            v.clear();
            // Keep capacity for reuse
        }
    )
    .build()
    .unwrap();
```

**Best for:**
- Expensive-to-create objects
- Objects with reusable resources
- Minimizing allocations within allocations

## Benchmarking Your Workload

### Use Criterion
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fastalloc::FixedPool;

fn benchmark_allocation(c: &mut Criterion) {
    let pool = FixedPool::<MyType>::new(1000).unwrap();
    
    c.bench_function("my_allocation", |b| {
        b.iter(|| {
            let handle = pool.allocate(black_box(MyType::new())).unwrap();
            black_box(&handle);
        });
    });
}

criterion_group!(benches, benchmark_allocation);
criterion_main!(benches);
```

### Measure Pool Overhead
Compare against standard allocation:

```rust
// Pool version
let pool = FixedPool::new(1000).unwrap();
let start = Instant::now();
for i in 0..1000 {
    let h = pool.allocate(i).unwrap();
    black_box(&h);
}
let pool_time = start.elapsed();

// Box version
let start = Instant::now();
for i in 0..1000 {
    let b = Box::new(i);
    black_box(&b);
}
let box_time = start.elapsed();

println!("Pool: {:?}, Box: {:?}, Speedup: {:.2}x",
         pool_time, box_time, box_time.as_nanos() as f64 / pool_time.as_nanos() as f64);
```

## Common Performance Issues

### Issue: Allocation Failures
```rust
// Bad: pool too small, constant failures
let pool = FixedPool::new(10).unwrap();
for i in 0..100 {
    let _ = pool.allocate(i); // Fails after 10
}

// Good: size appropriately
let pool = FixedPool::new(100).unwrap();
```

### Issue: Excessive Locking
```rust
// Bad: shared pool with high contention
let pool = Arc::new(ThreadSafePool::new(100).unwrap());
// Many threads competing

// Good: thread-local pools
thread_local! {
    static POOL: ThreadLocalPool<T> = ThreadLocalPool::new(100).unwrap();
}
```

### Issue: Frequent Growth
```rust
// Bad: starts too small, grows constantly
GrowthStrategy::Linear { amount: 1 }

// Good: reasonable initial size and growth
PoolConfig::builder()
    .capacity(1000)
    .growth_strategy(GrowthStrategy::Exponential { factor: 1.5 })
```

### Issue: Memory Waste
```rust
// Bad: huge pool, rarely used
let pool = FixedPool::new(1_000_000).unwrap();

// Good: growing pool with reasonable max
PoolConfig::builder()
    .capacity(100)
    .max_capacity(Some(10_000))
    .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
```

## Statistics-Guided Optimization

Enable the `stats` feature:

```rust
#[cfg(feature = "stats")]
{
    let stats = pool.statistics();
    
    println!("Peak usage: {} / {}", stats.peak_usage, stats.capacity);
    println!("Utilization: {:.1}%", stats.utilization_rate());
    println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    
    if stats.peak_usage < stats.capacity * 0.5 {
        println!("Consider reducing capacity");
    }
    
    if stats.hit_rate() < 0.95 {
        println!("Pool exhaustion detected, increase capacity");
    }
}
```

## Platform-Specific Tips

### Linux
- Use `parking_lot` feature for faster mutexes
- Consider huge pages for very large pools

### Windows
- Default std::Mutex is reasonably fast
- Monitor pool usage with performance counters

### Embedded / no_std
- Use FixedPool exclusively
- Disable all optional features
- Pre-initialize if possible to avoid runtime allocation

## Quick Wins Checklist

- [ ] Choose FixedPool if capacity is known
- [ ] Use ThreadLocalPool for single-threaded scenarios
- [ ] Enable `parking_lot` feature for thread-safe pools
- [ ] Set initial capacity based on typical usage
- [ ] Use exponential growth for unknown workloads
- [ ] Add cache-line alignment for shared data
- [ ] Implement reset functions for expensive types
- [ ] Measure with `stats` feature in development
- [ ] Benchmark your specific workload
- [ ] Profile to verify pool is the bottleneck

## Expected Performance Targets

### Fixed Pool
- Allocation: < 20ns
- Deallocation: < 10ns
- Memory overhead: < 2%

### Growing Pool  
- Allocation: < 50ns (steady state)
- Growth: < 100μs (for reasonable sizes)
- Memory overhead: < 5%

### Thread-Safe Pool
- Allocation: < 100ns (low contention)
- < 500ns (high contention)

If you're not meeting these targets, profile and investigate!
