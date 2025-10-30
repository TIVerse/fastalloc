# Error Handling Guide

## Pool Exhaustion Strategies

### Understanding Pool Exhaustion

Pool exhaustion occurs when you try to allocate from a pool that has no available slots.

**FixedPool**: Always returns error when full (cannot grow)  
**GrowingPool**: Tries to grow first, returns error if growth fails or max capacity reached  
**ThreadSafePool**: Same as GrowingPool (internally uses GrowingPool)

### Error Types

```rust
pub enum Error {
    // Pool has no available slots
    PoolExhausted {
        capacity: usize,
        allocated: usize,
    },
    
    // Growing pool hit its max capacity limit
    MaxCapacityExceeded {
        current: usize,
        requested: usize,
        max: usize,
    },
    
    // Configuration errors
    InvalidCapacity,
    InvalidAlignment { alignment: usize },
    // ... other errors
}
```

### Recommended Error Handling Patterns

#### Pattern 1: Fail Fast

```rust
use fastalloc::FixedPool;

let pool = FixedPool::<MyObject>::new(1000)?;

// If allocation fails, propagate error up the call stack
let handle = pool.allocate(my_object)?;
```

**When to use**: Critical resources where failure is unacceptable.

#### Pattern 2: Graceful Degradation

```rust
use fastalloc::FixedPool;

let pool = FixedPool::<MyObject>::new(1000)?;

match pool.allocate(my_object) {
    Ok(handle) => {
        // Use pooled object
        process_with_pool(handle);
    }
    Err(Error::PoolExhausted { .. }) => {
        // Fallback to heap allocation
        let boxed = Box::new(my_object);
        process_with_box(boxed);
    }
    Err(e) => return Err(e),
}
```

**When to use**: Non-critical paths where fallback is acceptable.

#### Pattern 3: Wait and Retry

```rust
use fastalloc::FixedPool;
use std::time::Duration;

let pool = FixedPool::<MyObject>::new(1000)?;

let handle = loop {
    match pool.try_allocate(my_object.clone()) {
        Some(handle) => break handle,
        None => {
            // Wait for other threads to return objects
            std::thread::sleep(Duration::from_millis(10));
            
            // Optional: Add timeout
            // if elapsed > MAX_WAIT { 
            //     return Err(Error::Timeout); 
            // }
        }
    }
};
```

**When to use**: Multi-threaded scenarios where objects are expected to be returned soon.

#### Pattern 4: Pre-allocation

```rust
use fastalloc::FixedPool;

// Calculate worst-case capacity upfront
let max_objects = calculate_peak_usage();
let pool = FixedPool::<MyObject>::new(max_objects)?;

// Now allocations won't fail (unless calculation was wrong)
let handle = pool.allocate(my_object).expect("pre-sized pool");
```

**When to use**: When peak usage is known or calculable.

#### Pattern 5: Growing Pool with Limits

```rust
use fastalloc::{GrowingPool, PoolConfig, GrowthStrategy};

let config = PoolConfig::builder()
    .capacity(100)
    .max_capacity(Some(10_000))  // Set reasonable limit
    .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
    .build()?;

let pool = GrowingPool::with_config(config)?;

match pool.allocate(my_object) {
    Ok(handle) => { /* use handle */ }
    Err(Error::MaxCapacityExceeded { max, .. }) => {
        log::error!("Pool hit max capacity of {}", max);
        // Handle gracefully or fail
    }
    Err(e) => return Err(e),
}
```

**When to use**: Variable workload with safety limits to prevent runaway memory usage.

### What Happens When Pool is Exhausted

**FixedPool**:
1. `allocate()` returns `Err(Error::PoolExhausted)`
2. Pool state unchanged (no side effects)
3. Caller must handle error

**GrowingPool**:
1. Attempts to grow using configured strategy
2. If growth succeeds, allocation proceeds normally
3. If growth fails (max capacity or allocation failure), returns error
4. If max capacity reached, returns `Err(Error::MaxCapacityExceeded)`

**ThreadSafePool**:
1. Acquires mutex
2. Follows GrowingPool logic
3. Releases mutex
4. Returns result

### Monitoring Pool Usage

#### Using Statistics (requires `stats` feature)

```rust
#[cfg(feature = "stats")]
{
    use fastalloc::FixedPool;
    
    let pool = FixedPool::<MyObject>::new(1000)?;
    
    // ... use pool ...
    
    let stats = pool.statistics();
    
    if stats.utilization_rate() > 0.9 {
        log::warn!("Pool is {}% full", stats.utilization_rate() * 100.0);
    }
    
    println!("Total allocations: {}", stats.total_allocations);
    println!("Current usage: {}/{}", stats.current_usage, stats.capacity);
}
```

#### Manual Tracking

```rust
use fastalloc::FixedPool;

let pool = FixedPool::<MyObject>::new(1000)?;

// Check before allocating
if pool.available() < 10 {
    log::warn!("Pool is nearly exhausted: {} slots left", pool.available());
}

match pool.allocate(my_object) {
    Ok(handle) => { /* use handle */ }
    Err(_) => { /* handle exhaustion */ }
}
```

### Best Practices

1. **Size pools appropriately**: Profile your application to determine actual usage
2. **Monitor in production**: Track pool utilization to detect capacity issues
3. **Set max limits on GrowingPool**: Prevent runaway memory growth
4. **Handle errors explicitly**: Don't unwrap() in production code
5. **Consider fallbacks**: Have a plan for when pool is exhausted
6. **Use separate pools per type**: Don't share pools across unrelated types
7. **Pre-allocate for critical paths**: Use FixedPool sized to peak usage

### Anti-Patterns to Avoid

❌ **Ignoring errors with unwrap()**:
```rust
let handle = pool.allocate(obj).unwrap(); // Will panic in production!
```

✅ **Handle errors properly**:
```rust
let handle = pool.allocate(obj)?; // or match
```

---

❌ **Undersizing FixedPool**:
```rust
let pool = FixedPool::new(10)?; // Way too small!
// Will fail frequently
```

✅ **Size appropriately**:
```rust
let pool = FixedPool::new(expected_peak * 1.2)?; // Add 20% buffer
```

---

❌ **Unbounded GrowingPool**:
```rust
let config = PoolConfig::builder()
    .capacity(10)
    .max_capacity(None) // Unlimited!
    .build()?;
```

✅ **Set reasonable limits**:
```rust
let config = PoolConfig::builder()
    .capacity(100)
    .max_capacity(Some(10_000)) // Safety limit
    .build()?;
```

---

❌ **Retrying indefinitely**:
```rust
loop {
    if let Some(handle) = pool.try_allocate(obj.clone()) {
        break handle;
    }
    // Infinite loop if pool never frees!
}
```

✅ **Add timeout**:
```rust
let start = Instant::now();
let handle = loop {
    if let Some(handle) = pool.try_allocate(obj.clone()) {
        break handle;
    }
    if start.elapsed() > Duration::from_secs(5) {
        return Err(Error::Timeout);
    }
    thread::sleep(Duration::from_millis(10));
};
```

## Error Recovery Strategies

### Strategy 1: Increase Pool Size

If you consistently hit pool exhaustion:

```rust
// Before
let pool = FixedPool::new(1000)?;

// After profiling, found peak usage is 1500
let pool = FixedPool::new(2000)?; // Add safety margin
```

### Strategy 2: Implement Object Lifecycle Management

```rust
use std::collections::VecDeque;

struct ObjectManager {
    pool: FixedPool<MyObject>,
    active: VecDeque<OwnedHandle<MyObject>>,
    max_active: usize,
}

impl ObjectManager {
    fn try_spawn(&mut self, obj: MyObject) -> Result<()> {
        // Enforce maximum active objects
        if self.active.len() >= self.max_active {
            // Remove oldest object to make room
            self.active.pop_front();
        }
        
        let handle = self.pool.allocate(obj)?;
        self.active.push_back(handle);
        Ok(())
    }
}
```

### Strategy 3: Prioritize Allocations

```rust
enum Priority {
    Critical,
    Normal,
    BestEffort,
}

fn allocate_with_priority<T>(
    pool: &FixedPool<T>,
    obj: T,
    priority: Priority,
) -> Result<OwnedHandle<T>> {
    match pool.allocate(obj) {
        Ok(handle) => Ok(handle),
        Err(Error::PoolExhausted { .. }) => {
            match priority {
                Priority::Critical => {
                    // Force allocation even if we have to drop something
                    // (application-specific logic)
                    Err(Error::PoolExhausted { /* ... */ })
                }
                Priority::Normal => {
                    Err(Error::PoolExhausted { /* ... */ })
                }
                Priority::BestEffort => {
                    // Just return error
                    Err(Error::PoolExhausted { /* ... */ })
                }
            }
        }
        Err(e) => Err(e),
    }
}
```

## Debugging Pool Exhaustion

### Enable Logging

```rust
use log::error;

match pool.allocate(obj) {
    Ok(handle) => Ok(handle),
    Err(Error::PoolExhausted { capacity, allocated }) => {
        error!(
            "Pool exhausted: {}/{} slots used",
            allocated, capacity
        );
        Err(Error::PoolExhausted { capacity, allocated })
    }
    Err(e) => Err(e),
}
```

### Track Allocation Patterns

```rust
#[cfg(feature = "stats")]
{
    let stats = pool.statistics();
    println!("Peak usage: {}", stats.peak_usage);
    println!("Avg usage: {:.1}", stats.average_usage());
    println!("Total allocs: {}", stats.total_allocations);
}
```

### Use Assertions in Development

```rust
#[cfg(debug_assertions)]
{
    assert!(
        pool.available() > 0,
        "Pool has {} available slots",
        pool.available()
    );
}
```

## Summary

- **FixedPool**: Returns error immediately when exhausted
- **GrowingPool**: Tries to grow first, errors if growth fails
- **Always handle errors**: Don't unwrap() in production
- **Monitor pool usage**: Use stats or manual tracking
- **Size appropriately**: Profile to determine actual needs
- **Set limits**: Prevent unbounded growth
- **Have fallback strategies**: Graceful degradation is better than crashes
