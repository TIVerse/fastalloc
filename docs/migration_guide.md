# Migration Guide

This guide helps you migrate from standard Rust allocation patterns to fastalloc.

## From Box

### Before

```rust
let object = Box::new(MyStruct::new());
// use object
drop(object);
```

### After

```rust
use fastalloc::FixedPool;

let pool = FixedPool::<MyStruct>::new(1000).unwrap();
let handle = pool.allocate(MyStruct::new()).unwrap();
// use handle (automatically derefs to MyStruct)
drop(handle); // Returns to pool
```

## From Vec for Object Collection

### Before

```rust
let mut entities: Vec<Box<Entity>> = Vec::new();

// Add entity
entities.push(Box::new(Entity::new()));

// Remove entity
entities.retain(|e| e.is_alive());
```

### After

```rust
use fastalloc::FixedPool;

let pool = FixedPool::<Entity>::new(10000).unwrap();
let mut entities = Vec::new();

// Add entity
entities.push(pool.allocate(Entity::new()).unwrap());

// Remove entity - automatically returns to pool
entities.retain(|e| e.is_alive());
```

## From Arena Allocators

### Before (typed-arena)

```rust
use typed_arena::Arena;

let arena = Arena::new();
let obj = arena.alloc(MyStruct::new());
// Can't free individual objects
// Everything freed when arena drops
```

### After

```rust
use fastalloc::FixedPool;

let pool = FixedPool::<MyStruct>::new(1000).unwrap();
let handle = pool.allocate(MyStruct::new()).unwrap();
// Can free individual objects
drop(handle); // Freed immediately
```

## Game Engine Integration

### Before (manual pooling)

```rust
struct EntityManager {
    entities: Vec<Option<Entity>>,
    free_indices: Vec<usize>,
}

impl EntityManager {
    fn spawn(&mut self, entity: Entity) -> usize {
        if let Some(index) = self.free_indices.pop() {
            self.entities[index] = Some(entity);
            index
        } else {
            self.entities.push(Some(entity));
            self.entities.len() - 1
        }
    }
    
    fn despawn(&mut self, index: usize) {
        self.entities[index] = None;
        self.free_indices.push(index);
    }
}
```

### After

```rust
use fastalloc::FixedPool;

struct EntityManager {
    pool: FixedPool<Entity>,
}

impl EntityManager {
    fn new(capacity: usize) -> Self {
        Self {
            pool: FixedPool::new(capacity).unwrap(),
        }
    }
    
    fn spawn(&self, entity: Entity) -> Result<OwnedHandle<Entity>> {
        self.pool.allocate(entity)
    }
    
    // despawn is automatic when handle is dropped
}
```

## Server Connection Pooling

### Before

```rust
use std::sync::Mutex;

struct ConnectionPool {
    available: Mutex<Vec<Connection>>,
}

impl ConnectionPool {
    fn acquire(&self) -> Option<Connection> {
        self.available.lock().unwrap().pop()
    }
    
    fn release(&self, conn: Connection) {
        self.available.lock().unwrap().push(conn);
    }
}
```

### After

```rust
use fastalloc::ThreadSafePool;
use std::sync::Arc;

struct ConnectionPool {
    pool: Arc<ThreadSafePool<Connection>>,
}

impl ConnectionPool {
    fn new(size: usize) -> Self {
        Self {
            pool: Arc::new(ThreadSafePool::new(size).unwrap()),
        }
    }
    
    fn acquire(&self, conn: Connection) -> Result<OwnedHandle<Connection>> {
        self.pool.allocate(conn)
    }
    
    // release is automatic when handle is dropped
}
```

## Particle Systems

### Before

```rust
struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    fn update(&mut self, dt: f32) {
        self.particles.retain_mut(|p| {
            p.lifetime -= dt;
            p.lifetime > 0.0
        });
    }
    
    fn emit(&mut self, particle: Particle) {
        self.particles.push(particle);
    }
}
```

### After

```rust
use fastalloc::FixedPool;

struct ParticleSystem {
    pool: FixedPool<Particle>,
    particles: Vec<OwnedHandle<Particle>>,
}

impl ParticleSystem {
    fn new(max_particles: usize) -> Self {
        Self {
            pool: FixedPool::new(max_particles).unwrap(),
            particles: Vec::new(),
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.particles.retain_mut(|p| {
            p.lifetime -= dt;
            p.lifetime > 0.0
            // Dead particles automatically return to pool
        });
    }
    
    fn emit(&mut self, particle: Particle) {
        if let Ok(handle) = self.pool.allocate(particle) {
            self.particles.push(handle);
        }
    }
}
```

## Common Patterns

### Pattern: Initialization with Cleanup

Before:
```rust
impl Drop for MyType {
    fn drop(&mut self) {
        // cleanup
    }
}
```

After:
```rust
use fastalloc::Poolable;

impl Poolable for MyType {
    fn on_acquire(&mut self) {
        // initialize/reset
    }
    
    fn on_release(&mut self) {
        // cleanup before returning to pool
    }
}
```

### Pattern: Growing Collections

Before:
```rust
let mut items = Vec::new();
// Grows automatically
```

After:
```rust
use fastalloc::{GrowingPool, PoolConfig, GrowthStrategy};

let config = PoolConfig::builder()
    .capacity(100)
    .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
    .build()
    .unwrap();
let pool = GrowingPool::with_config(config).unwrap();
```

## Performance Comparison

Run benchmarks to verify improvements:

```bash
cargo bench
```

Typical speedups:
- Allocation: 5-10x faster
- Deallocation: 2-5x faster
- Cache performance: 20-30% improvement

## Gotchas and Tips

### 1. Pool Capacity

**Problem:** Pool exhaustion errors
```rust
let pool = FixedPool::new(10).unwrap();
// ... allocate 11 items -> Error!
```

**Solution:** Size appropriately or use GrowingPool
```rust
let config = PoolConfig::builder()
    .capacity(10)
    .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
    .build()
    .unwrap();
let pool = GrowingPool::with_config(config).unwrap();
```

### 2. Handle Lifetimes

Handles are tied to pool lifetime:
```rust
let handle = {
    let pool = FixedPool::new(10).unwrap();
    pool.allocate(42).unwrap()
}; // Error: pool dropped, handle invalid
```

Solution: Keep pool alive longer than handles.

### 3. Thread Safety

**Wrong:** Sharing FixedPool across threads
```rust
let pool = Arc::new(FixedPool::new(100).unwrap()); // Won't compile - FixedPool is !Sync
```

**Right:** Use ThreadSafePool
```rust
let pool = Arc::new(ThreadSafePool::new(100).unwrap());
```

### 4. Statistics Overhead

Enable only in development:
```toml
[dev-dependencies]
fastalloc = { version = "1.0", features = ["stats"] }
```

## Checklist

- [ ] Identified hot allocation paths
- [ ] Measured baseline performance
- [ ] Chose appropriate pool type
- [ ] Sized pool capacity based on profiling
- [ ] Updated error handling for pool exhaustion
- [ ] Configured growth strategy if needed
- [ ] Added tests for pool edge cases
- [ ] Benchmarked to verify improvements
- [ ] Documented pool usage in code

## Need Help?

- Check examples: `cargo run --example basic_usage`
- Read API docs: https://docs.rs/fastalloc
- Open an issue: https://github.com/TIVerse/fastalloc/issues
