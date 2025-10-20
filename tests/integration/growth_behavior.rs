//! Tests for pool growth behavior.

use fastalloc::{GrowingPool, GrowthStrategy, PoolConfig};

#[test]
fn test_linear_growth() {
    let config = PoolConfig::builder()
        .capacity(10)
        .growth_strategy(GrowthStrategy::Linear { amount: 5 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    assert_eq!(pool.capacity(), 10);
    
    // Fill initial capacity
    let mut handles = Vec::new();
    for i in 0..10 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    // Trigger growth
    handles.push(pool.allocate(10).unwrap());
    assert_eq!(pool.capacity(), 15);
    
    // Trigger another growth
    for i in 11..16 {
        handles.push(pool.allocate(i).unwrap());
    }
    assert_eq!(pool.capacity(), 20);
}

#[test]
fn test_exponential_growth() {
    let config = PoolConfig::builder()
        .capacity(10)
        .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    
    // Fill initial capacity
    for i in 0..10 {
        handles.push(pool.allocate(i).unwrap());
    }
    assert_eq!(pool.capacity(), 10);
    
    // Trigger growth - should double
    handles.push(pool.allocate(10).unwrap());
    assert_eq!(pool.capacity(), 20);
    
    // Fill to trigger next growth
    for i in 11..21 {
        handles.push(pool.allocate(i).unwrap());
    }
    assert_eq!(pool.capacity(), 40);
}

#[test]
fn test_custom_growth() {
    let config = PoolConfig::builder()
        .capacity(10)
        .growth_strategy(GrowthStrategy::Custom {
            compute: Box::new(|current| current / 2),
        })
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..10 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    // Should grow by 10/2 = 5
    handles.push(pool.allocate(10).unwrap());
    assert_eq!(pool.capacity(), 15);
}

#[test]
fn test_no_growth_behaves_like_fixed() {
    let config = PoolConfig::builder()
        .capacity(5)
        .growth_strategy(GrowthStrategy::None)
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..5 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    assert_eq!(pool.capacity(), 5);
    assert!(pool.is_full());
    
    // Should fail to allocate
    let result = pool.allocate(5);
    assert!(result.is_err());
}

#[test]
fn test_max_capacity_limit() {
    let config = PoolConfig::builder()
        .capacity(5)
        .max_capacity(Some(10))
        .growth_strategy(GrowthStrategy::Linear { amount: 10 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    for i in 0..5 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    // Try to grow - but should be capped at max_capacity
    for i in 5..10 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    assert_eq!(pool.capacity(), 10); // Capped at max
    
    // Further allocations should fail
    let result = pool.allocate(10);
    assert!(result.is_err());
}

#[test]
fn test_growth_with_deallocation() {
    let config = PoolConfig::builder()
        .capacity(5)
        .growth_strategy(GrowthStrategy::Linear { amount: 5 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    
    // Fill and grow
    for i in 0..10 {
        handles.push(pool.allocate(i).unwrap());
    }
    assert_eq!(pool.capacity(), 10);
    
    // Free some
    handles.drain(0..5);
    assert_eq!(pool.allocated(), 5);
    assert_eq!(pool.available(), 5);
    
    // Should reuse freed slots before growing
    for i in 10..15 {
        handles.push(pool.allocate(i).unwrap());
    }
    
    assert_eq!(pool.capacity(), 10); // Shouldn't have grown yet
    assert_eq!(pool.allocated(), 10);
}

#[test]
fn test_multiple_growth_cycles() {
    let config = PoolConfig::builder()
        .capacity(2)
        .max_capacity(Some(100))
        .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
        .build()
        .unwrap();
    
    let pool = GrowingPool::<i32>::with_config(config).unwrap();
    
    let mut handles = Vec::new();
    let expected_capacities = vec![2, 4, 8, 16, 32, 64];
    
    for (cycle, &expected_cap) in expected_capacities.iter().enumerate() {
        // Fill to capacity
        while pool.allocated() < expected_cap {
            handles.push(pool.allocate(cycle as i32).unwrap());
        }
        
        assert_eq!(pool.capacity(), expected_cap);
    }
}
