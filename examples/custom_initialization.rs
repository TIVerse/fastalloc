//! Custom initialization and configuration example.

use fastalloc::{GrowingPool, GrowthStrategy, PoolConfig};

fn main() {
    println!("=== Custom Initialization Example ===\n");

    // Example 1: Custom initializer
    println!("1. Pool with Custom Initializer:");

    let config = PoolConfig::builder()
        .capacity(10)
        .max_capacity(Some(50))
        .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
        .alignment(64) // Cache-line aligned
        .initializer(|| {
            println!("  [Init] Creating new object");
            String::from("initialized")
        })
        .build()
        .expect("Invalid configuration");

    let pool = GrowingPool::with_config(config).expect("Failed to create pool");

    println!("Pool created with custom initializer");
    println!(
        "Capacity: {}, Available: {}\n",
        pool.capacity(),
        pool.available()
    );

    // Example 2: Custom reset function
    println!("2. Pool with Reset Function:");

    #[derive(Debug, Clone)]
    struct Buffer {
        data: Vec<u8>,
        position: usize,
    }

    impl fastalloc::Poolable for Buffer {}

    let config = PoolConfig::builder()
        .capacity(5)
        .reset_fn(
            || Buffer {
                data: Vec::with_capacity(1024),
                position: 0,
            },
            |buffer| {
                // Reset buffer when returned to pool
                buffer.data.clear();
                buffer.position = 0;
                println!("  [Reset] Buffer cleaned");
            },
        )
        .build()
        .expect("Invalid configuration");

    let buffer_pool = GrowingPool::with_config(config).expect("Failed to create pool");

    {
        let mut buf = buffer_pool
            .allocate(Buffer {
                data: vec![1, 2, 3, 4, 5],
                position: 5,
            })
            .unwrap();

        println!("Buffer allocated: {:?}", *buf);
        buf.data.extend_from_slice(&[6, 7, 8]);
        println!("Buffer modified: {:?}", *buf);
    } // Buffer returned to pool and reset function is called

    println!();

    // Example 3: Different growth strategies
    println!("3. Growth Strategies:");

    // Linear growth
    let linear_config = PoolConfig::builder()
        .capacity(10)
        .growth_strategy(GrowthStrategy::Linear { amount: 5 })
        .build()
        .unwrap();

    let linear_pool = GrowingPool::<i32>::with_config(linear_config).unwrap();
    println!(
        "Linear growth pool: initial capacity = {}",
        linear_pool.capacity()
    );

    // Exponential growth
    let exp_config = PoolConfig::builder()
        .capacity(10)
        .growth_strategy(GrowthStrategy::Exponential { factor: 2.0 })
        .build()
        .unwrap();

    let exp_pool = GrowingPool::<i32>::with_config(exp_config).unwrap();
    println!(
        "Exponential growth pool: initial capacity = {}",
        exp_pool.capacity()
    );

    // Custom growth
    let custom_config = PoolConfig::builder()
        .capacity(10)
        .growth_strategy(GrowthStrategy::Custom {
            compute: Box::new(|current| {
                // Grow by 50% rounded up
                (current as f32 * 1.5).ceil() as usize
            }),
        })
        .build()
        .unwrap();

    let custom_pool = GrowingPool::<i32>::with_config(custom_config).unwrap();
    println!(
        "Custom growth pool: initial capacity = {}",
        custom_pool.capacity()
    );

    println!("\n=== Example Complete ===");
}
