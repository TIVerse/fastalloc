//! Tests for thread-safe pool operations.

use fastalloc::{ThreadSafePool, ThreadLocalPool, PoolConfig};
use std::sync::Arc;
use std::thread;

#[test]
fn test_thread_safe_pool_basic() {
    let pool = ThreadSafePool::<i32>::new(100).unwrap();
    
    let handle = pool.allocate(42).unwrap();
    assert_eq!(*handle, 42);
}

#[test]
fn test_thread_safe_pool_concurrent_allocation() {
    let pool = Arc::new(ThreadSafePool::<i32>::new(1000).unwrap());
    
    let mut threads = vec![];
    
    for thread_id in 0..10 {
        let pool_clone = Arc::clone(&pool);
        threads.push(thread::spawn(move || {
            let mut handles = Vec::new();
            for i in 0..50 {
                handles.push(pool_clone.allocate(thread_id * 100 + i).unwrap());
            }
            handles.len()
        }));
    }
    
    let mut total = 0;
    for t in threads {
        total += t.join().unwrap();
    }
    
    assert_eq!(total, 500);
}

#[test]
fn test_thread_safe_pool_concurrent_alloc_dealloc() {
    let pool = Arc::new(ThreadSafePool::<i32>::new(100).unwrap());
    
    let mut threads = vec![];
    
    for thread_id in 0..4 {
        let pool_clone = Arc::clone(&pool);
        threads.push(thread::spawn(move || {
            for i in 0..100 {
                let handle = pool_clone.allocate(thread_id * 1000 + i).unwrap();
                drop(handle); // Immediately deallocate
            }
        }));
    }
    
    for t in threads {
        t.join().unwrap();
    }
    
    // All handles should be returned
    assert_eq!(pool.available(), pool.capacity());
}

#[test]
fn test_thread_local_pool_basic() {
    let pool = ThreadLocalPool::<i32>::new(50).unwrap();
    
    let handle = pool.allocate(42).unwrap();
    assert_eq!(*handle, 42);
}

#[test]
fn test_thread_local_pool_moved_to_thread() {
    let pool = ThreadLocalPool::<String>::new(10).unwrap();
    
    let handle = thread::spawn(move || {
        // Pool was moved here
        let h = pool.allocate(String::from("Hello")).unwrap();
        h.clone()
    });
    
    let result = handle.join().unwrap();
    assert_eq!(*result, "Hello");
}

#[test]
fn test_thread_local_pool_per_thread() {
    let mut threads = vec![];
    
    for thread_id in 0..5 {
        threads.push(thread::spawn(move || {
            // Each thread creates its own pool
            let pool = ThreadLocalPool::<i32>::new(20).unwrap();
            
            let mut sum = 0;
            for i in 0..10 {
                let handle = pool.allocate(thread_id * 10 + i).unwrap();
                sum += *handle;
            }
            
            sum
        }));
    }
    
    let mut total = 0;
    for t in threads {
        total += t.join().unwrap();
    }
    
    assert!(total > 0);
}

#[test]
fn test_thread_safe_pool_cloning() {
    let pool1 = Arc::new(ThreadSafePool::<i32>::new(50).unwrap());
    let pool2 = Arc::clone(&pool1);
    
    let h1 = pool1.allocate(42).unwrap();
    let h2 = pool2.allocate(99).unwrap();
    
    assert_eq!(*h1, 42);
    assert_eq!(*h2, 99);
    
    // Both refer to the same pool
    assert_eq!(pool1.allocated(), 2);
    assert_eq!(pool2.allocated(), 2);
}

#[test]
fn test_concurrent_stress() {
    let pool = Arc::new(ThreadSafePool::<Vec<u8>>::new(200).unwrap());
    
    let mut threads = vec![];
    
    for _ in 0..8 {
        let pool_clone = Arc::clone(&pool);
        threads.push(thread::spawn(move || {
            for _ in 0..1000 {
                if let Ok(handle) = pool_clone.allocate(vec![0u8; 64]) {
                    drop(handle);
                }
            }
        }));
    }
    
    for t in threads {
        t.join().unwrap();
    }
    
    // Should be fully available after all threads complete
    assert_eq!(pool.available(), pool.capacity());
}
