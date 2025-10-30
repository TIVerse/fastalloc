# Benchmark Results

## Test Environment

- **CPU**: (varies by system - run `cargo bench` to get results for your hardware)
- **Rust Version**: 1.70+
- **Optimization**: Release mode with LTO enabled
- **Methodology**: Using criterion.rs with 100 samples per benchmark

## Allocation Performance (Latest Results)

### Fixed Pool Allocation (i32)

| Pool Size | Time per Allocation | Throughput |
|-----------|---------------------|------------|
| 100       | 3.47 ns            | 28.8 Gelem/s |
| 1,000     | 3.44 ns            | 290.8 Gelem/s |
| 10,000    | 3.59 ns            | 2782 Gelem/s |

### Growing Pool Allocation (i32)

| Pool Size | Time per Allocation | Throughput |
|-----------|---------------------|------------|
| 100       | 4.63 ns            | 21.6 Gelem/s |
| 1,000     | 4.54 ns            | 220.3 Gelem/s |
| 10,000    | 4.57 ns            | 2190 Gelem/s |

### Box Allocation Baseline (i32)

| Pool Size | Time per Allocation | Throughput |
|-----------|---------------------|------------|
| 100       | 4.80 ns            | 20.8 Gelem/s |
| 1,000     | 4.82 ns            | 207.6 Gelem/s |
| 10,000    | 4.85 ns            | 2061 Gelem/s |

## Performance Comparison

**FixedPool vs Box**: ~1.3-1.4x faster (28% improvement)  
**GrowingPool vs Box**: ~1.05x faster (5% improvement)

### Notes on Performance Claims

1. **Modern Allocators are Fast**: The system allocator (jemalloc/mimalloc) is highly optimized on modern systems, typically taking 5-10ns per allocation for small objects with good cache locality.

2. **Pool Advantages**:
   - **Predictable latency**: Pools eliminate worst-case allocation spikes
   - **Cache locality**: Objects are stored contiguously 
   - **Reduced fragmentation**: No heap fragmentation over time
   - **Deterministic performance**: Critical for real-time systems

3. **Where Pools Excel**:
   - High allocation/deallocation churn (creating/destroying objects in tight loops)
   - Real-time systems requiring bounded latency
   - Embedded systems with limited heap
   - Avoiding memory fragmentation in long-running processes

4. **Where Standard Allocator Works Well**:
   - Mixed object sizes
   - Unpredictable allocation patterns
   - Long-lived objects

## Type Size Impact

| Type | Size | Time per Allocation |
|------|------|---------------------|
| u8 | 1 byte | 3.66 ns |
| u128 | 16 bytes | 3.63 ns |
| Large struct | 256 bytes | 11.86 ns |

**Observation**: Allocation time increases with object size due to initialization overhead.

## Allocation Reuse Pattern

**Time per allocate-drop-reallocate cycle**: 7.25 ns

This demonstrates the LIFO (Last-In-First-Out) cache-friendly behavior of the stack allocator.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench allocation_speed

# Save results
cargo bench -- --save-baseline my-baseline

# Compare with baseline
cargo bench -- --baseline my-baseline
```

## Benchmark Interpretation

### What the Numbers Mean

- **Time per allocation**: Average time to allocate a single object
- **Throughput**: Number of allocations per second
- **Lower is better** for time, **higher is better** for throughput

### Caveats

1. Microbenchmarks may not reflect real-world performance
2. Your mileage may vary based on:
   - CPU architecture and cache size
   - System allocator implementation
   - Memory pressure and fragmentation
   - Compiler version and optimizations

### Real-World Benefits

The primary benefits of memory pools are NOT raw speed but:

1. **Predictable latency** (no GC pauses, no heap fragmentation spikes)
2. **Memory locality** (better cache utilization)
3. **Deterministic behavior** (critical for real-time systems)
4. **Reduced system calls** (batch allocation)

## Comparison with Other Pooling Libraries

### Feature Comparison

| Library | Handle-based RAII | Deallocation | Thread-Safe | no_std | Growth | Use Case |
|---------|-------------------|--------------|-------------|--------|---------|----------|
| **fastalloc** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | General pooling with handles |
| **typed-arena** | ❌ No | ❌ No | ❌ No | ✅ Yes | ✅ Yes | Arena allocation (bulk drop) |
| **bumpalo** | ❌ No | ❌ No | ❌ No | ✅ Yes | ✅ Yes | Bump allocator (append-only) |
| **slotmap** | ⚠️ Keys | ✅ Yes | ❌ No | ✅ Yes | ✅ Yes | Stable generational indices |
| **sharded-slab** | ⚠️ Refs | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes | Lock-free concurrent slab |

### When to Use Each

**fastalloc**: 
- Need RAII automatic return to pool
- Mixed allocation/deallocation patterns
- Want thread-safe option
- Need growing pools with configurable strategies

**typed-arena**: 
- All objects same lifetime (bulk drop)
- Append-only until arena is cleared
- Maximum performance for arena pattern
- Don't need individual deallocation

**bumpalo**: 
- Bump allocation pattern
- No individual deallocation needed
- Want to reset entire allocator
- Optimized for append-only

**slotmap**: 
- Need stable indices that survive reallocation
- Generational indices prevent ABA problem
- Want to refer to objects by ID
- Don't need handles with Deref

**sharded-slab**: 
- High-concurrency scenarios
- Need lock-free allocation
- Can tolerate refs instead of owned handles
- Only need std support

### Performance Comparison (Rough Estimates)

| Operation | fastalloc | typed-arena | bumpalo | slotmap |
|-----------|-----------|-------------|---------|---------|
| Allocate | ~3.5ns | ~2ns | ~1ns | ~5ns |
| Deallocate | ~3.5ns | N/A | N/A | ~5ns |
| Lookup | Direct | Direct | Direct | ~2ns |
| Thread-safe | ~100ns | N/A | N/A | Varies |

**Note**: These are rough estimates. Actual performance depends on usage patterns, cache behavior, and system architecture. Always benchmark your specific workload.

## Methodology

All benchmarks use:
- Criterion.rs for statistical rigor
- Black box to prevent compiler optimizations
- Warm-up period to stabilize caches
- 100 samples with outlier detection

## Contributing Benchmarks

We welcome benchmark contributions! Please include:
1. Your system specifications
2. Benchmark methodology
3. Reproducible code
4. Real-world use case description
