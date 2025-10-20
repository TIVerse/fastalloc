use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fastalloc::FixedPool;

fn bench_pool_vs_box(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_vs_box");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::new("pool", size), size, |b, &size| {
            let pool = FixedPool::<i32>::new(size).unwrap();
            b.iter(|| {
                let h = pool.allocate(black_box(42)).unwrap();
                black_box(&h);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("box", size), size, |b, _size| {
            b.iter(|| {
                let b = Box::new(black_box(42));
                black_box(&b);
            });
        });
    }
    
    group.finish();
}

fn bench_pool_vs_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_vs_vec_reserve");
    
    for size in [100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("pool", size), size, |b, &size| {
            let pool = FixedPool::<i32>::new(size).unwrap();
            b.iter(|| {
                let mut handles = Vec::with_capacity(size);
                for i in 0..size {
                    handles.push(pool.allocate(i as i32).unwrap());
                }
                black_box(&handles);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("vec", size), size, |b, &size| {
            b.iter(|| {
                let mut vec = Vec::with_capacity(size);
                for i in 0..size {
                    vec.push(Box::new(i as i32));
                }
                black_box(&vec);
            });
        });
    }
    
    group.finish();
}

fn bench_struct_allocation(c: &mut Criterion) {
    #[derive(Clone)]
    struct GameObject {
        position: (f32, f32, f32),
        velocity: (f32, f32, f32),
        health: i32,
        team: u8,
    }
    
    let mut group = c.benchmark_group("struct_allocation");
    
    group.bench_function("pool", |b| {
        let pool = FixedPool::<GameObject>::new(1000).unwrap();
        b.iter(|| {
            let obj = GameObject {
                position: (1.0, 2.0, 3.0),
                velocity: (0.1, 0.2, 0.3),
                health: 100,
                team: 1,
            };
            let h = pool.allocate(black_box(obj)).unwrap();
            black_box(&h);
        });
    });
    
    group.bench_function("box", |b| {
        b.iter(|| {
            let obj = GameObject {
                position: (1.0, 2.0, 3.0),
                velocity: (0.1, 0.2, 0.3),
                health: 100,
                team: 1,
            };
            let b = Box::new(black_box(obj));
            black_box(&b);
        });
    });
    
    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    
    // Sequential access in pool (good cache locality)
    group.bench_function("pool_sequential", |b| {
        let pool = FixedPool::<i32>::new(1000).unwrap();
        let mut handles = Vec::new();
        for i in 0..1000 {
            handles.push(pool.allocate(i).unwrap());
        }
        
        b.iter(|| {
            let mut sum = 0;
            for h in &handles {
                sum += **h;
            }
            black_box(sum);
        });
    });
    
    // Vec access (also good cache locality)
    group.bench_function("vec_sequential", |b| {
        let vec: Vec<i32> = (0..1000).collect();
        
        b.iter(|| {
            let mut sum = 0;
            for v in &vec {
                sum += *v;
            }
            black_box(sum);
        });
    });
    
    // Boxed values (potentially poor cache locality)
    group.bench_function("box_vector_sequential", |b| {
        let boxes: Vec<Box<i32>> = (0..1000).map(|i| Box::new(i)).collect();
        
        b.iter(|| {
            let mut sum = 0;
            for b in &boxes {
                sum += **b;
            }
            black_box(sum);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_pool_vs_box,
    bench_pool_vs_vec,
    bench_struct_allocation,
    bench_cache_performance
);
criterion_main!(benches);
