use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fastalloc::{PoolConfig, ThreadSafePool};
use std::sync::Arc;
use std::thread;

fn bench_thread_safe_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_safe_pool");
    
    let thread_counts = [1, 2, 4, 8];
    
    for &threads in &thread_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_allocation", threads),
            &threads,
            |b, &threads| {
                let config = PoolConfig::builder()
                    .capacity(10000)
                    .build()
                    .unwrap();
                let pool = Arc::new(ThreadSafePool::with_config(config).unwrap());
                
                b.iter(|| {
                    let mut handles = vec![];
                    
                    for _ in 0..threads {
                        let pool_clone = Arc::clone(&pool);
                        handles.push(thread::spawn(move || {
                            for i in 0..100 {
                                if let Ok(handle) = pool_clone.allocate(black_box(i)) {
                                    black_box(handle);
                                }
                            }
                        }));
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("contention");
    
    // High contention scenario: small pool, many threads
    group.bench_function("high_contention", |b| {
        let config = PoolConfig::builder()
            .capacity(50)
            .build()
            .unwrap();
        let pool = Arc::new(ThreadSafePool::with_config(config).unwrap());
        
        b.iter(|| {
            let mut handles = vec![];
            
            for _ in 0..8 {
                let pool_clone = Arc::clone(&pool);
                handles.push(thread::spawn(move || {
                    for i in 0..10 {
                        if let Ok(handle) = pool_clone.allocate(black_box(i)) {
                            black_box(&handle);
                            // Hold briefly then release
                            drop(handle);
                        }
                    }
                }));
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    // Low contention scenario: large pool, few threads
    group.bench_function("low_contention", |b| {
        let config = PoolConfig::builder()
            .capacity(10000)
            .build()
            .unwrap();
        let pool = Arc::new(ThreadSafePool::with_config(config).unwrap());
        
        b.iter(|| {
            let mut handles = vec![];
            
            for _ in 0..2 {
                let pool_clone = Arc::clone(&pool);
                handles.push(thread::spawn(move || {
                    for i in 0..100 {
                        if let Ok(handle) = pool_clone.allocate(black_box(i)) {
                            black_box(&handle);
                        }
                    }
                }));
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.finish();
}

fn bench_thread_local_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_local_pool");
    
    group.bench_function("single_thread", |b| {
        use fastalloc::ThreadLocalPool;
        
        let pool = ThreadLocalPool::<i32>::new(1000).unwrap();
        
        b.iter(|| {
            for i in 0..100 {
                if let Ok(handle) = pool.allocate(black_box(i)) {
                    black_box(handle);
                }
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_thread_safe_pool,
    bench_contention,
    bench_thread_local_pool
);
criterion_main!(benches);
