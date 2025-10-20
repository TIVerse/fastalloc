//! Integration tests for basic pool operations.

use fastalloc::{FixedPool, GrowingPool, PoolConfig, GrowthStrategy};

#[test]
fn test_fixed_pool_basic() {
    let pool = FixedPool::<i32>::new(10).unwrap();
    
    assert_eq!(pool.capacity(), 10);
    assert_eq!(pool.available(), 10);
    assert_eq!(pool.allocated(), 0);
}

#[test]
fn test_allocate_and_drop() {
    let pool = FixedPool::new(5).unwrap();
    
    {
        let h1 = pool.allocate(42).unwrap();
        assert_eq!(*h1, 42);
        assert_eq!(pool.allocated(), 1);
    }
    
    assert_eq!(pool.allocated(), 0);
}

#[test]
fn test_multiple_allocations() {
    let pool = FixedPool::new(10).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..5 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    assert_eq!(pool.allocated(), 5);
    assert_eq!(pool.available(), 5);
    
    for (i, handle) in handles.iter().enumerate() {
        assert_eq!(**handle, i as i32);
    }
}

#[test]
fn test_pool_exhaustion() {
    let pool = FixedPool::new(3).unwrap();
    
    let _h1 = pool.allocate(1).unwrap();
    let _h2 = pool.allocate(2).unwrap();
    let _h3 = pool.allocate(3).unwrap();
    
    assert!(pool.is_full());
    
    let result = pool.allocate(4);
    assert!(result.is_err());
}

#[test]
fn test_reuse_after_drop() {
    let pool = FixedPool::new(2).unwrap();
    
    {
        let _h1 = pool.allocate(1).unwrap();
        let _h2 = pool.allocate(2).unwrap();
        assert!(pool.is_full());
    }
    
    assert!(pool.is_empty());
    
    let _h3 = pool.allocate(3).unwrap();
    assert_eq!(pool.allocated(), 1);
}

#[test]
fn test_modify_allocated_value() {
    let pool = FixedPool::new(5).unwrap();
    
    let mut handle = pool.allocate(10).unwrap();
    assert_eq!(*handle, 10);
    
    *handle = 20;
    assert_eq!(*handle, 20);
    
    *handle += 5;
    assert_eq!(*handle, 25);
}

#[test]
fn test_growing_pool() {
    let config = PoolConfig::builder()
        .capacity(5)
        .growth_strategy(GrowthStrategy::Linear { amount: 5 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::with_config(config).unwrap();
    
    assert_eq!(pool.capacity(), 5);
    
    let mut handles = Vec::new();
    for i in 0..10 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    assert_eq!(pool.capacity(), 10);
    assert_eq!(pool.allocated(), 10);
}

#[test]
fn test_growing_pool_max_capacity() {
    let config = PoolConfig::builder()
        .capacity(2)
        .max_capacity(Some(5))
        .growth_strategy(GrowthStrategy::Linear { amount: 2 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..5 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    assert_eq!(pool.capacity(), 6); // 2 + 2 + 2
    
    // Should fail to exceed max capacity
    handles.push(pool.allocate(5).unwrap());
    let result = pool.allocate(6);
    assert!(result.is_err());
}

#[test]
fn test_struct_allocation() {
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }
    
    let pool = FixedPool::<Point>::new(10).unwrap();
    
    let mut p = pool.allocate(Point { x: 1, y: 2 }).unwrap();
    assert_eq!(p.x, 1);
    assert_eq!(p.y, 2);
    
    p.x = 10;
    p.y = 20;
    assert_eq!(*p, Point { x: 10, y: 20 });
}

#[test]
fn test_string_allocation() {
    let pool = FixedPool::<String>::new(5).unwrap();
    
    let mut s = pool.allocate(String::from("Hello")).unwrap();
    assert_eq!(*s, "Hello");
    
    s.push_str(", World!");
    assert_eq!(*s, "Hello, World!");
}

#[test]
fn test_vec_allocation() {
    let pool = FixedPool::<Vec<i32>>::new(5).unwrap();
    
    let mut v = pool.allocate(vec![1, 2, 3]).unwrap();
    assert_eq!(*v, vec![1, 2, 3]);
    
    v.push(4);
    assert_eq!(*v, vec![1, 2, 3, 4]);
}
