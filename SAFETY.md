# Safety Documentation

## Overview

`fastalloc` prioritizes memory safety while providing high-performance memory pooling. This document explains the safety guarantees, `unsafe` usage, and invariants maintained by the library.

## Safety Guarantees

### Type Safety

All pool operations are type-safe and prevent the following at compile time:
- Type confusion (wrong type retrieved from pool)
- Lifetime violations (dangling references)
- Double-free (handled by ownership system)

### Memory Safety

The library prevents:
- ✅ **Use-after-free**: Handles maintain exclusive ownership
- ✅ **Double-free**: Allocator tracks which slots are in use
- ✅ **Memory leaks**: RAII ensures objects are returned to pool
- ✅ **Data races**: Thread-safe types use proper synchronization

## Handle Safety

### OwnedHandle

```rust
pub struct OwnedHandle<'pool, T> {
    pool: &'pool dyn PoolInterface<T>,
    index: usize,
}
```

**Safety invariants**:
1. The handle holds a lifetime-bound reference to the pool
2. The index is guaranteed valid for the pool's lifetime
3. Drop automatically returns the object to the pool (RAII)
4. Only one handle can exist per allocated slot

**Why it's safe**:
- Rust's borrow checker prevents the pool from being dropped while handles exist
- Interior mutability (RefCell) ensures mutable access is exclusive
- The allocator prevents reusing a slot while a handle exists

### ThreadSafeHandle

```rust
pub struct ThreadSafeHandle<T> {
    pool: Arc<Mutex<GrowingPool<T>>>,
    index: usize,
    cached_ptr: *mut T,
}
```

**Safety invariants**:
1. The pool is behind `Arc<Mutex>` preventing concurrent mutation
2. The cached pointer is valid for the handle's lifetime
3. The pointer is never dereferenced after the handle is dropped
4. Only this handle can access the cached pointer

**Why it's safe**:
- Pool storage never moves (uses stable Vec storage per chunk)
- Allocator ensures slot isn't reused while handle exists
- Mutex prevents concurrent access during allocation/deallocation
- Pointer is obtained under lock and remains valid until drop

**Thread safety**:
- `Send`: Can be sent across threads (pool is behind Arc)
- NOT `Sync`: Cannot be shared between threads (contains raw pointer)

## Unsafe Code Usage

### Where `unsafe` is Used

1. **Pool storage access** (`FixedPool::get`, `GrowingPool::get`)
   - Extends lifetime beyond RefCell borrow
   - **Safe because**: Pool owns storage, index is valid, handle prevents reuse

2. **Mutable reference from immutable receiver** (`get_mut`)
   - Returns `&mut T` from `&self`
   - **Safe because**: RefCell/Mutex provides interior mutability, handle ensures exclusivity

3. **MaybeUninit access** (allocation and deallocation)
   - Writing to uninitialized memory
   - **Safe because**: Write happens before any read, drop called before slot reuse

4. **Pointer casting** (storage access)
   - `*const MaybeUninit<T>` → `*const T`
   - **Safe because**: Memory is initialized by allocate() before handle creation

### Unsafe Code Review

All `unsafe` blocks include:
- Safety comments explaining invariants
- Documentation of assumptions
- Reasoning about why the operation is sound

## Alignment Requirements

### Default Alignment

Types use their natural alignment (`mem::align_of::<T>()`).

### Custom Alignment

```rust
let config = PoolConfig::builder()
    .alignment(64)  // Cache-line aligned
    .build()?;
```

**Requirements**:
- Alignment must be a power of 2
- Alignment must be ≥ natural alignment of T
- Validated at config build time

**Safety**: Rust's allocator guarantees proper alignment when allocation size is a multiple of alignment.

## Drop Behavior

### Automatic Cleanup

When a handle is dropped:
1. `Poolable::on_release()` is called (user-defined cleanup)
2. `drop_in_place()` runs the destructor
3. Allocator marks slot as free
4. Statistics updated (if enabled)

### Pool Destruction

When a pool is dropped:
- Remaining allocated objects are dropped
- Memory is deallocated by Vec's drop
- No manual cleanup required

### Panic Safety

If a panic occurs:
- Handle drop will still run (RAII guarantee)
- Object is properly returned to pool
- No memory leaks occur

## Concurrency Safety

### FixedPool / GrowingPool

**NOT thread-safe**:
- Uses `RefCell` for interior mutability
- Will panic if accessed concurrently

**Use case**: Single-threaded or thread-local usage

### ThreadSafePool

**Thread-safe**:
- Uses `Mutex` (or `parking_lot::Mutex`)
- Safe for concurrent access from multiple threads
- May block waiting for lock

**Trade-off**: Locks add overhead (~100ns per operation)

### ThreadLocalPool

**Per-thread safety**:
- Each thread has its own pool
- No synchronization overhead
- Cannot share handles between threads

## Memory Layout

### FixedPool Storage

```
[MaybeUninit<T>; capacity]
     ↓
Storage never moves after creation
Pointers remain valid for pool lifetime
```

### GrowingPool Storage

```
Vec<Vec<MaybeUninit<T>>>
     ↓
Chunk 0: [MaybeUninit<T>; initial_capacity]
Chunk 1: [MaybeUninit<T>; growth_amount]
Chunk 2: [MaybeUninit<T>; growth_amount]
```

**Important**: Individual chunks never move (Vec of Vecs), so pointers remain stable.

## Debug Assertions

In debug mode, additional checks are enabled:

1. **Double-free detection**: Bitmap tracks allocated slots
2. **Index bounds checking**: Validates indices on every operation
3. **Allocation state tracking**: Ensures consistency

**Release mode**: These checks are compiled out for performance.

## Common Pitfalls and How We Avoid Them

### Pitfall 1: Use-After-Free

**Problem**: Accessing memory after it's been freed.

**Prevention**: 
- Handles borrow the pool preventing it from being dropped
- RAII ensures objects are returned only when handle is dropped
- Borrow checker prevents multiple mutable references

### Pitfall 2: Memory Leaks

**Problem**: Forgetting to return objects to the pool.

**Prevention**:
- Handle Drop impl automatically returns objects
- No manual `free()` call required
- Rust's affine type system prevents forgetting

### Pitfall 3: Data Races

**Problem**: Concurrent access to mutable state.

**Prevention**:
- Non-thread-safe pools use RefCell (panics on concurrent access)
- Thread-safe pools use Mutex (blocks concurrent access)
- Type system enforces Send/Sync bounds

### Pitfall 4: Pointer Invalidation

**Problem**: Pointers becoming invalid after reallocation.

**Prevention**:
- Storage is stable (doesn't move)
- Chunks in GrowingPool are individually allocated
- Pointers are cached only while handle exists

## Testing for Safety

Run comprehensive tests:

```bash
# Standard tests
cargo test

# Miri (undefined behavior detection)
cargo +nightly miri test

# Address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# Thread sanitizer  
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test
```

## Reporting Safety Issues

If you discover a safety issue:
1. **DO NOT** open a public GitHub issue
2. Email security concerns to: [maintainer email]
3. Include:
   - Minimal reproduction
   - Rust version and target
   - Expected vs actual behavior

## Future Safety Improvements

Planned enhancements:
- [ ] Formal verification of core unsafe code
- [ ] Fuzzing with cargo-fuzz
- [ ] Static analysis with MIRAI
- [ ] Memory model validation with loom

## References

- [Rustonomicon: Unsafe Rust](https://doc.rust-lang.org/nomicon/)
- [Rust Memory Model](https://github.com/rust-lang/unsafe-code-guidelines)
- [API Guidelines: Unsafe](https://rust-lang.github.io/api-guidelines/necessities.html#unsafe-functions-have-a-safety-section-c-safety)
