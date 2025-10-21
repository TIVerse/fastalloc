//! Basic usage example demonstrating core fastalloc features.

use fastalloc::{FixedPool, GrowingPool, GrowthStrategy, PoolConfig};

fn main() {
    println!("=== FastAlloc Basic Usage Example ===\n");

    // Example 1: Simple fixed pool
    println!("1. Fixed Pool:");
    let pool = FixedPool::<i32>::new(10).expect("Failed to create pool");

    println!("   Created pool with capacity: {}", pool.capacity());
    println!("   Available slots: {}", pool.available());

    // Allocate some values
    let mut handle1 = pool.allocate(42).expect("Allocation failed");
    println!("   Allocated: {}", *handle1);

    // Modify the value
    *handle1 = 100;
    println!("   Modified to: {}", *handle1);

    let handle2 = pool.allocate(200).expect("Allocation failed");
    println!("   Allocated another: {}", *handle2);
    println!("   Available slots now: {}\n", pool.available());

    // Values are automatically returned to pool when handles are dropped
    drop(handle1);
    drop(handle2);
    println!(
        "   After dropping handles, available: {}\n",
        pool.available()
    );

    // Example 2: Growing pool with configuration
    println!("2. Growing Pool:");
    let config = PoolConfig::builder()
        .capacity(5)
        .max_capacity(Some(20))
        .growth_strategy(GrowthStrategy::Linear { amount: 5 })
        .build()
        .expect("Invalid configuration");

    let growing_pool = GrowingPool::with_config(config).expect("Failed to create pool");

    println!("   Initial capacity: {}", growing_pool.capacity());

    let mut handles = Vec::new();

    // Allocate beyond initial capacity - pool will grow
    for i in 0..15 {
        handles.push(growing_pool.allocate(i).expect("Allocation failed"));
    }

    println!("   After allocating 15 items:");
    println!("   Current capacity: {}", growing_pool.capacity());
    println!("   Allocated: {}", growing_pool.allocated());
    println!("   Available: {}\n", growing_pool.available());

    // Example 3: Working with structs
    println!("3. Struct Allocation:");

    #[derive(Debug)]
    struct Point {
        x: f32,
        y: f32,
    }

    impl fastalloc::Poolable for Point {}

    let point_pool = FixedPool::<Point>::new(100).expect("Failed to create pool");

    let mut point = point_pool
        .allocate(Point { x: 10.0, y: 20.0 })
        .expect("Allocation failed");

    println!("   Allocated point: {:?}", *point);

    point.x = 50.0;
    point.y = 75.0;

    println!("   Modified point: {:?}\n", *point);

    // Example 4: Demonstrating reuse
    println!("4. Memory Reuse:");
    let reuse_pool = FixedPool::<String>::new(3).expect("Failed to create pool");

    {
        let h1 = reuse_pool.allocate(String::from("First")).unwrap();
        let h2 = reuse_pool.allocate(String::from("Second")).unwrap();
        println!(
            "   Allocated 2 strings, available: {}",
            reuse_pool.available()
        );
        drop(h1); // Explicitly drop to show reuse
    }

    println!(
        "   After dropping first handle, available: {}",
        reuse_pool.available()
    );

    let h3 = reuse_pool.allocate(String::from("Third")).unwrap();
    println!(
        "   Allocated new string (reused slot), available: {}",
        reuse_pool.available()
    );

    println!("\n=== Example Complete ===");
}
