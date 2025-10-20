# Architecture

## Overview

`fastalloc` is designed as a modular memory pooling library with a focus on performance, type safety, and flexibility.

## Core Components

### 1. Pool Implementations

#### FixedPool
- Pre-allocated storage with fixed capacity
- Uses `StackAllocator` for O(1) LIFO allocation
- Zero fragmentation
- Best performance characteristics
- No runtime growth

#### GrowingPool
- Dynamic capacity with configurable growth strategies
- Uses `FreeListAllocator` for efficient slot management
- Grows in chunks based on strategy
- Supports max capacity limits
- Slightly higher overhead than FixedPool

#### ThreadLocalPool
- Wrapper around FixedPool
- Per-thread instances
- Zero synchronization overhead
- Not Sync (prevents sharing between threads)
- Ideal for single-threaded or thread-affine workloads

#### ThreadSafePool
- Lock-based concurrent access
- Uses `Mutex` (std) or `parking_lot::Mutex`
- Suitable for moderate contention scenarios
- Thread-safe handles with RAII

#### LockFreePool (experimental)
- Atomic operations for allocation
- Uses `crossbeam` data structures
- Best performance under high contention
- Requires `lock-free` feature

### 2. Allocation Strategies

All allocators implement the `Allocator` trait:

```rust
trait Allocator {
    fn allocate(&mut self) -> Option<usize>;
    fn free(&mut self, index: usize);
    fn available(&self) -> usize;
    fn capacity(&self) -> usize;
}
```

#### StackAllocator (LIFO)
- Push/pop stack of free indices
- Excellent cache locality
- O(1) allocation and deallocation
- Simple and fast
- Used by: `FixedPool`

#### FreeListAllocator
- Vector-based free list
- O(1) allocation and deallocation
- Good for random access patterns
- Used by: `GrowingPool`

#### BitmapAllocator
- Bit-packed allocation tracking
- Minimal memory overhead (1 bit per slot)
- O(n) worst case for allocation (with hint optimization)
- Space-efficient for large pools
- Used by: specialized scenarios

### 3. Handle System

Handles provide RAII-based automatic return to pool:

#### OwnedHandle
- Exclusive ownership
- `Deref` and `DerefMut` for value access
- Drop returns object to pool
- Cannot be cloned (maintains exclusivity)

#### SharedHandle
- Reference-counted ownership
- Multiple handles to same object
- Returns to pool when last reference drops
- Uses `Rc` internally (not thread-safe)

#### WeakHandle
- Non-owning reference
- Can be upgraded to SharedHandle
- Doesn't prevent pool return
- Useful for caching/observation

### 4. Configuration System

Builder pattern for type-safe configuration:

```rust
PoolConfig::builder()
    .capacity(1000)
    .max_capacity(Some(10000))
    .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
    .alignment(64)
    .pre_initialize(false)
    .build()?
```

Validates all parameters before pool creation.

### 5. Statistics Collection

Optional compile-time feature (`stats`):

- `StatisticsCollector`: Tracks metrics during pool operations
- `PoolStatistics`: Snapshot of current state
- `StatisticsReporter`: Formats for output/monitoring
- Zero overhead when disabled

## Data Flow

### Allocation Flow

```
User Request
    ↓
Pool.allocate(value)
    ↓
Allocator.allocate() → index
    ↓
Write value to storage[index]
    ↓
Create Handle(pool, index)
    ↓
Return Handle to user
```

### Deallocation Flow

```
Handle dropped
    ↓
Handle::drop()
    ↓
Pool.return_to_pool(index)
    ↓
Call value.on_release()
    ↓
Drop value
    ↓
Allocator.free(index)
    ↓
Slot available for reuse
```

### Growth Flow (GrowingPool)

```
Allocation request
    ↓
Allocator empty?
    ↓ yes
Check growth allowed?
    ↓ yes
Compute growth amount
    ↓
Check max capacity
    ↓
Allocate new chunk
    ↓
Extend allocator
    ↓
Retry allocation
```

## Memory Layout

### FixedPool Storage

```
Storage: Vec<MaybeUninit<T>>
[Slot 0][Slot 1][Slot 2]...[Slot N-1]

Allocator: StackAllocator
Free stack: [N-1, N-2, ..., 2, 1, 0]
```

Contiguous memory → excellent cache locality

### GrowingPool Storage

```
Storage: Vec<Vec<MaybeUninit<T>>>
Chunk 0: [Slot 0][Slot 1]...[Slot C0-1]
Chunk 1: [Slot C0][Slot C0+1]...[Slot C0+C1-1]
...

Allocator: FreeListAllocator
Free list: [5, 2, 8, 12, ...]
```

Multiple chunks → some indirection overhead

## Safety Guarantees

### Type Safety
- Generic over `T` with compile-time size checks
- No type punning or unsafe casts in public API
- Lifetime-bound handles prevent dangling references

### Memory Safety
- `MaybeUninit<T>` for uninitialized storage
- Careful initialization and drop management
- Debug assertions for double-free detection
- RefCell for interior mutability (non-thread-safe pools)
- Mutex for synchronization (thread-safe pools)

### Thread Safety
- `Send` bounds where appropriate
- `Sync` only for explicitly thread-safe types
- Clear !Sync markers for thread-local pools

## Performance Considerations

### Cache Efficiency
- Contiguous storage in FixedPool
- Stack allocator promotes temporal locality
- Alignment support for cache-line optimization

### Allocation Speed
- O(1) operations in common case
- No syscalls after pool creation
- Predictable performance (no GC pauses)

### Memory Overhead
- Per-slot: 1-8 bytes (depends on allocator)
- Plus: allocator metadata (small, constant)
- Growing pools: pointer indirection overhead

## Extension Points

### Custom Growth Strategies
```rust
GrowthStrategy::Custom {
    compute: Box::new(|current| { /* custom logic */ })
}
```

### Custom Initialization
```rust
InitializationStrategy::Custom {
    initializer: Box::new(|| { /* create */ }),
    reset: Some(Box::new(|obj| { /* reset */ }))
}
```

### Poolable Trait
```rust
impl Poolable for MyType {
    fn on_acquire(&mut self) { /* setup */ }
    fn on_release(&mut self) { /* cleanup */ }
}
```

## Future Enhancements

- GPU memory pools
- Distributed pools across machines
- Automatic sizing from profiling
- Async runtime integration
- Custom drop strategies
- Defragmentation utilities
