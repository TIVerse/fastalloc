use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fastalloc::{FixedPool, GrowingPool, PoolConfig};

fn bench_fixed_pool_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixed_pool_allocation");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let pool = FixedPool::<i32>::new(size).unwrap();
            b.iter(|| {
                let handle = pool.allocate(black_box(42)).unwrap();
                black_box(handle);
            });
        });
    }
    
    group.finish();
}

fn bench_growing_pool_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("growing_pool_allocation");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let config = PoolConfig::builder()
                .capacity(size)
                .growth_strategy(fastalloc::GrowthStrategy::Exponential { factor: 2.0 })
                .build()
                .unwrap();
            let pool = GrowingPool::with_config(config).unwrap();
            
            b.iter(|| {
                let handle = pool.allocate(black_box(42)).unwrap();
                black_box(handle);
            });
        });
    }
    
    group.finish();
}

fn bench_box_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("box_allocation_baseline");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _size| {
            b.iter(|| {
                let boxed = Box::new(black_box(42));
                black_box(boxed);
            });
        });
    }
    
    group.finish();
}

fn bench_allocation_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_reuse");
    
    let pool = FixedPool::<i32>::new(100).unwrap();
    
    group.bench_function("reuse_pattern", |b| {
        b.iter(|| {
            let handle = pool.allocate(black_box(42)).unwrap();
            black_box(&handle);
            drop(handle);
            let handle = pool.allocate(black_box(99)).unwrap();
            black_box(&handle);
        });
    });
    
    group.finish();
}

fn bench_different_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_by_type_size");
    
    // Small type
    group.bench_function("u8", |b| {
        let pool = FixedPool::<u8>::new(1000).unwrap();
        b.iter(|| {
            let handle = pool.allocate(black_box(42u8)).unwrap();
            black_box(handle);
        });
    });
    
    // Medium type
    group.bench_function("u128", |b| {
        let pool = FixedPool::<u128>::new(1000).unwrap();
        b.iter(|| {
            let handle = pool.allocate(black_box(42u128)).unwrap();
            black_box(handle);
        });
    });
    
    // Large type
    #[derive(Clone)]
    struct LargeStruct([u64; 32]);
    
    group.bench_function("large_struct_256_bytes", |b| {
        let pool = FixedPool::<LargeStruct>::new(1000).unwrap();
        b.iter(|| {
            let handle = pool.allocate(black_box(LargeStruct([0; 32]))).unwrap();
            black_box(handle);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_fixed_pool_allocation,
    bench_growing_pool_allocation,
    bench_box_allocation,
    bench_allocation_reuse,
    bench_different_sizes
);
criterion_main!(benches);
