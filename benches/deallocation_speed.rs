use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fastalloc::FixedPool;

fn bench_deallocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("deallocation");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("pool", size), size, |b, &size| {
            let pool = FixedPool::<i32>::new(size).unwrap();
            
            b.iter(|| {
                let handle = pool.allocate(black_box(42)).unwrap();
                drop(black_box(handle));
            });
        });
        
        group.bench_with_input(BenchmarkId::new("box", size), size, |b, _size| {
            b.iter(|| {
                let boxed = Box::new(black_box(42));
                drop(black_box(boxed));
            });
        });
    }
    
    group.finish();
}

fn bench_bulk_deallocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_deallocation");
    
    let sizes = [10, 100, 1000];
    
    for &size in &sizes {
        group.bench_with_input(BenchmarkId::new("pool", size), &size, |b, &size| {
            let pool = FixedPool::<i32>::new(size).unwrap();
            
            b.iter(|| {
                let mut handles = Vec::with_capacity(size);
                for i in 0..size {
                    handles.push(pool.allocate(i as i32).unwrap());
                }
                
                // Deallocation happens when handles vector is dropped
                drop(black_box(handles));
            });
        });
        
        group.bench_with_input(BenchmarkId::new("box", size), &size, |b, &size| {
            b.iter(|| {
                let mut boxes = Vec::with_capacity(size);
                for i in 0..size {
                    boxes.push(Box::new(i as i32));
                }
                
                drop(black_box(boxes));
            });
        });
    }
    
    group.finish();
}

fn bench_mixed_allocation_deallocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_alloc_dealloc");
    
    group.bench_function("pool_churning", |b| {
        let pool = FixedPool::<i32>::new(100).unwrap();
        
        b.iter(|| {
            for i in 0..50 {
                let h1 = pool.allocate(i).unwrap();
                let h2 = pool.allocate(i + 1).unwrap();
                drop(h1);
                let h3 = pool.allocate(i + 2).unwrap();
                drop(h2);
                drop(h3);
            }
        });
    });
    
    group.bench_function("box_churning", |b| {
        b.iter(|| {
            for i in 0..50 {
                let h1 = Box::new(i);
                let h2 = Box::new(i + 1);
                drop(h1);
                let h3 = Box::new(i + 2);
                drop(h2);
                drop(h3);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_deallocation,
    bench_bulk_deallocation,
    bench_mixed_allocation_deallocation
);
criterion_main!(benches);
