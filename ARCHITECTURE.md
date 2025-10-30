# Architecture Documentation

## Overview

This document explains the internal architecture of fastalloc, including design decisions, data structures, and implementation details.

## Core Components

### 1. Pool Types

#### FixedPool

**Purpose**: Pre-allocated fixed-size pool with O(1) operations.

**Structure**:
```rust
pub struct FixedPool<T> {
    storage: RefCell<Vec<MaybeUninit<T>>>,
    allocator: RefCell<StackAllocator>,
    capacity: usize,
    config: PoolConfig<T>,
}
```

**Design rationale**:
- `Vec<MaybeUninit<T>>`: Allows uninitialized memory until explicitly allocated
- `RefCell`: Provides interior mutability for single-threaded use
- `StackAllocator`: LIFO allocation for cache-friendly access patterns

**Trade-offs**:
- ✅ Fastest allocation (~3.5ns)
- ✅ Zero fragmentation
- ✅ Predictable memory usage
- ❌ Fixed capacity (cannot grow)
- ❌ Not thread-safe

#### GrowingPool

**Purpose**: Dynamic pool that grows on demand.

**Structure**:
```rust
pub struct GrowingPool<T> {
    storage: RefCell<Vec<Vec<MaybeUninit<T>>>>,
    allocator: RefCell<FreeListAllocator>,
    capacity: RefCell<usize>,
    chunk_boundaries: RefCell<Vec<usize>>,
    config: PoolConfig<T>,
}
```

**Design rationale**:
- `Vec<Vec<...>>`: Chunks never move, making pointers stable
- `FreeListAllocator`: Better for random access than stack
- `chunk_boundaries`: Cached for O(log n) binary search instead of O(n) linear scan

**Growth strategy**:
- Linear: Add fixed amount each time
- Exponential: Multiply by factor (e.g., 2x)
- Custom: User-defined function

**Trade-offs**:
- ✅ Automatic growth
- ✅ No fixed capacity limit
- ✅ Stable pointers across growth
- ❌ Slightly slower (~4.6ns)
- ❌ More complex memory layout

#### ThreadSafePool

**Purpose**: Thread-safe pool for concurrent access.

**Structure**:
```rust
pub struct ThreadSafePool<T> {
    inner: Arc<Mutex<GrowingPool<T>>>,
}

pub struct ThreadSafeHandle<T> {
    pool: Arc<Mutex<GrowingPool<T>>>,
    index: usize,
    cached_ptr: *mut T,  // Key optimization!
}
```

**Key optimization**: Cached pointer eliminates lock acquisition on every dereference.

**Design rationale**:
- `Arc<Mutex>`: Shared ownership with exclusive access
- Cached pointer: Lock-free deref after initial allocation
- Only locks during allocate/deallocate

**Trade-offs**:
- ✅ Thread-safe
- ✅ Lock-free deref (10-50x improvement)
- ✅ Works with multiple threads
- ❌ Mutex overhead on allocate/free
- ❌ Cannot be Sync (contains raw pointer)

### 2. Allocators

#### StackAllocator (LIFO)

**Strategy**: Last-In-First-Out using a stack of free indices.

```rust
struct StackAllocator {
    free_stack: Vec<usize>,
    capacity: usize,
    allocated_bitmap: Vec<u64>,  // Debug only
}
```

**Allocation**: `O(1)` - Pop from stack  
**Deallocation**: `O(1)` - Push to stack

**Advantages**:
- Excellent cache locality (recently freed = hot in cache)
- Simple implementation
- Optimal for allocate/free patterns

**Used by**: `FixedPool`

#### FreeListAllocator

**Strategy**: Maintains list of free slot indices.

```rust
struct FreeListAllocator {
    free_list: Vec<usize>,
    capacity: usize,
    allocated_bitmap: Vec<u64>,  // Debug only
}
```

**Allocation**: `O(1)` - Pop from list  
**Deallocation**: `O(1)` - Push to list

**Advantages**:
- Good for random access patterns
- Can extend capacity dynamically
- No preference for recently freed slots

**Used by**: `GrowingPool`

#### BitmapAllocator

**Strategy**: Bit vector tracking allocation status.

```rust
struct BitmapAllocator {
    bitmap: Vec<u64>,
    capacity: usize,
    allocated: usize,
    next_free_hint: usize,
}
```

**Allocation**: `O(n)` worst case, `O(1)` typical (with hint)  
**Deallocation**: `O(1)`

**Advantages**:
- Minimal memory overhead (1 bit per slot)
- Fast intrinsics (trailing_zeros)
- Hint tracking for common patterns

**Used by**: Optional, for space-constrained scenarios

### 3. Handles

#### OwnedHandle

**Purpose**: RAII-based exclusive ownership of pool object.

```rust
pub struct OwnedHandle<'pool, T> {
    pool: &'pool dyn PoolInterface<T>,
    index: usize,
}
```

**Lifetime safety**:
- `'pool` lifetime prevents pool from being dropped
- Borrow checker ensures exclusive access
- Drop impl automatically returns object

**Trait implementations**:
- `Deref` / `DerefMut`: Direct access to value
- `Drop`: Returns object to pool
- `Debug`, `Display`, `PartialEq`, `Ord`: For convenience

#### SharedHandle

**Purpose**: Reference-counted shared access.

```rust
pub struct SharedHandle<'pool, T> {
    inner: Rc<OwnedHandle<'pool, T>>,
}
```

**Use case**: Multiple readers need access to same pooled object.

**Trade-off**: Reference counting overhead vs flexibility.

#### WeakHandle

**Purpose**: Non-owning reference that may become invalid.

```rust
pub struct WeakHandle<'pool, T> {
    inner: Weak<OwnedHandle<'pool, T>>,
}
```

**Use case**: Breaking reference cycles, observer patterns.

## Memory Layout

### FixedPool Memory Layout

```
FixedPool<T>
├── storage: Vec<MaybeUninit<T>>
│   ├── [0]: MaybeUninit<T>  ← Object or uninitialized
│   ├── [1]: MaybeUninit<T>
│   └── [n]: MaybeUninit<T>
├── allocator: StackAllocator
│   └── free_stack: [n, n-1, ..., 2, 1, 0]  ← Available indices
└── capacity: n
```

**Key properties**:
- Contiguous memory block
- Fixed size, never moves
- Pointers stable for pool lifetime

### GrowingPool Memory Layout

```
GrowingPool<T>
├── storage: Vec<Vec<MaybeUninit<T>>>
│   ├── Chunk 0: [MaybeUninit<T>; initial_cap]
│   ├── Chunk 1: [MaybeUninit<T>; growth_amount]
│   └── Chunk 2: [MaybeUninit<T>; growth_amount]
├── chunk_boundaries: [initial_cap, initial_cap + g1, initial_cap + g1 + g2]
├── allocator: FreeListAllocator
└── capacity: total across chunks
```

**Index to pointer mapping**:
```rust
fn compute_chunk_location(index: usize) -> (chunk_idx, offset) {
    // Binary search in chunk_boundaries
    chunk_idx = binary_search(chunk_boundaries, index + 1)
    offset = if chunk_idx == 0 { 
        index 
    } else { 
        index - chunk_boundaries[chunk_idx - 1] 
    }
}
```

**Key properties**:
- Chunks never move individually
- Pointers remain valid across growth
- O(log n) lookup with cached boundaries

## Unsafe Code Justification

### Pattern 1: Extending RefCell Borrow Lifetime

```rust
pub(crate) fn get(&self, index: usize) -> &T {
    let storage = self.storage.borrow();
    unsafe {
        let ptr = storage.as_ptr();
        &*ptr.add(index).cast::<T>()
    }
}
```

**Why unsafe?**: Returns reference that outlives the borrow.

**Why safe?**:
1. Pool owns the storage (won't be freed)
2. Index is valid (checked by allocator)
3. Handle prevents concurrent mutable access
4. Memory is initialized before handle creation

### Pattern 2: Mutable from Immutable

```rust
pub(crate) fn get_mut(&self, index: usize) -> &mut T {
    let storage = self.storage.borrow_mut();
    unsafe {
        let ptr = storage.as_ptr() as *mut MaybeUninit<T>;
        &mut *ptr.add(index).cast::<T>()
    }
}
```

**Why unsafe?**: `&mut T` from `&self`.

**Why safe?**:
1. RefCell provides interior mutability
2. Only one handle per slot exists
3. Borrow checker ensures no aliasing via handle

### Pattern 3: MaybeUninit Access

```rust
// Allocation
storage[index].write(value);  // Initialize
Ok(OwnedHandle::new(self, index))  // Now safe to read

// Deallocation  
let value_ptr = storage[index].as_mut_ptr();
(*value_ptr).on_release();
ptr::drop_in_place(value_ptr);  // Drop before reuse
```

**Why safe?**:
1. Write before any read
2. Drop before reuse
3. Handle ensures no access after drop

## Performance Optimizations

### 1. Cached Chunk Boundaries (GrowingPool)

**Before**: O(n) linear scan through chunks  
**After**: O(log n) binary search

**Implementation**:
```rust
chunk_boundaries: RefCell<Vec<usize>>  // [100, 200, 400, 800]
```

Updated on growth, used in `compute_chunk_location`.

### 2. Lock-Free Deref (ThreadSafeHandle)

**Before**: Lock on every dereference  
**After**: Cached pointer, lock only on alloc/dealloc

**Impact**: 10-50x improvement for handle access patterns

### 3. Bitmap Double-Free Check

**Before**: O(n) `contains()` in FreeListAllocator  
**After**: O(1) bitmap check

**Impact**: Eliminates debug build overhead for large pools

### 4. Inline Annotations

Critical path functions marked with `#[inline]`:
- Allocator methods
- Handle deref/deref_mut
- Poolable trait hooks

### 5. Pre-allocation with Capacity

```rust
let mut free_stack = Vec::with_capacity(capacity);
for i in (0..capacity).rev() {
    free_stack.push(i);
}
```

**Better than**: `(0..capacity).rev().collect()` (avoids reallocation)

## Testing Strategy

### Unit Tests

- **Allocator tests**: Verify allocation/deallocation logic
- **Pool tests**: Test capacity, growth, reuse
- **Handle tests**: RAII behavior, drop semantics
- **Safety tests**: Borrow checker violations (compile failures)

### Integration Tests

- Multi-threaded scenarios
- Edge cases (capacity limits, exhaustion)
- Drop behavior and cleanup

### Benchmark Tests

- Criterion.rs for statistical rigor
- Comparison with Box allocation
- Various pool sizes and types

### Property-Based Testing (Future)

- QuickCheck/proptest for invariant checking
- Fuzz testing with cargo-fuzz

## Design Principles

1. **Safety First**: Leverage Rust's type system, minimal unsafe
2. **Zero-Cost Abstractions**: Performance comparable to hand-written code
3. **Ergonomic API**: RAII handles, builder pattern, sensible defaults
4. **Composability**: Multiple pool types for different use cases
5. **Transparency**: Well-documented unsafe code with justification

## Future Improvements

- [ ] Lock-free data structures (experimental `lock-free` feature)
- [ ] NUMA-aware allocation for large systems
- [ ] Custom allocator backend support
- [ ] Allocation batching for reduced overhead
- [ ] Per-thread caching for thread-safe pool

## References

- [Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [The Rust Reference](https://doc.rust-lang.org/reference/)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Memory pool design patterns from C/C++ literature
