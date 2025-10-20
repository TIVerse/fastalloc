//! Edge case tests for pool implementations.

use fastalloc::{FixedPool, GrowingPool, PoolConfig, GrowthStrategy, Error};

#[test]
fn test_zero_capacity_rejected() {
    let result = FixedPool::<i32>::new(0);
    assert!(result.is_err());
}

#[test]
fn test_single_element_pool() {
    let pool = FixedPool::<i32>::new(1).unwrap();
    
    assert_eq!(pool.capacity(), 1);
    
    let h1 = pool.allocate(42).unwrap();
    assert_eq!(*h1, 42);
    assert!(pool.is_full());
    
    let result = pool.allocate(99);
    assert!(result.is_err());
    
    drop(h1);
    assert!(pool.is_empty());
}

#[test]
fn test_large_capacity() {
    let pool = FixedPool::<u8>::new(1_000_000).unwrap();
    assert_eq!(pool.capacity(), 1_000_000);
    assert_eq!(pool.available(), 1_000_000);
}

#[test]
fn test_zst_allocation() {
    #[derive(Debug, PartialEq)]
    struct ZeroSized;
    
    let pool = FixedPool::<ZeroSized>::new(10).unwrap();
    
    let h1 = pool.allocate(ZeroSized).unwrap();
    let h2 = pool.allocate(ZeroSized).unwrap();
    
    assert_eq!(*h1, ZeroSized);
    assert_eq!(*h2, ZeroSized);
}

#[test]
fn test_invalid_alignment() {
    let result = PoolConfig::<i32>::builder()
        .capacity(10)
        .alignment(7) // Not a power of 2
        .build();
    
    assert!(result.is_err());
}

#[test]
fn test_max_capacity_less_than_capacity() {
    let result = PoolConfig::<i32>::builder()
        .capacity(100)
        .max_capacity(Some(50))
        .build();
    
    assert!(result.is_err());
}

#[test]
fn test_rapid_alloc_dealloc() {
    let pool = FixedPool::<i32>::new(10).unwrap();
    
    for i in 0..1000 {
        let handle = pool.allocate(i).unwrap();
        drop(handle);
    }
    
    assert!(pool.is_empty());
}

#[test]
fn test_allocation_order_independence() {
    let pool = FixedPool::<i32>::new(5).unwrap();
    
    let h1 = pool.allocate(1).unwrap();
    let h2 = pool.allocate(2).unwrap();
    let h3 = pool.allocate(3).unwrap();
    
    drop(h2); // Drop middle handle
    drop(h1); // Drop first handle
    
    let h4 = pool.allocate(4).unwrap();
    let h5 = pool.allocate(5).unwrap();
    
    assert_eq!(*h3, 3);
    assert_eq!(*h4, 4);
    assert_eq!(*h5, 5);
}

#[test]
fn test_growing_pool_exact_max_capacity() {
    let config = PoolConfig::builder()
        .capacity(5)
        .max_capacity(Some(5))
        .growth_strategy(GrowthStrategy::Linear { amount: 10 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..5 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    assert_eq!(pool.capacity(), 5);
    
    // Should not be able to grow
    let result = pool.allocate(5);
    assert!(result.is_err());
}

#[test]
fn test_none_growth_strategy() {
    let config = PoolConfig::builder()
        .capacity(3)
        .growth_strategy(GrowthStrategy::None)
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..3 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    let result = pool.allocate(3);
    assert!(matches!(result, Err(Error::PoolExhausted { .. })));
}

#[test]
fn test_drop_order_doesnt_matter() {
    let pool = FixedPool::<String>::new(5).unwrap();
    
    let h1 = pool.allocate(String::from("one")).unwrap();
    let h2 = pool.allocate(String::from("two")).unwrap();
    let h3 = pool.allocate(String::from("three")).unwrap();
    
    // Drop in different order
    drop(h3);
    drop(h1);
    drop(h2);
    
    assert_eq!(pool.available(), 5);
}

#[test]
fn test_handle_comparison() {
    let pool = FixedPool::<i32>::new(5).unwrap();
    
    let h1 = pool.allocate(42).unwrap();
    let h2 = pool.allocate(42).unwrap();
    let h3 = pool.allocate(99).unwrap();
    
    assert_eq!(h1, h2); // Same value
    assert_ne!(h1, h3); // Different value
}

#[test]
fn test_large_object_allocation() {
    struct LargeObject {
        data: [u8; 1024 * 10], // 10 KB
    }
    
    let pool = FixedPool::<LargeObject>::new(10).unwrap();
    
    let obj = LargeObject { data: [0; 1024 * 10] };
    let handle = pool.allocate(obj).unwrap();
    
    assert_eq!(handle.data[0], 0);
}
