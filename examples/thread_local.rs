//! Thread-local pool example demonstrating zero-synchronization performance.

use fastalloc::ThreadLocalPool;
use std::thread;

fn main() {
    println!("=== Thread-Local Pool Example ===\n");

    // Example 1: Single thread usage
    println!("1. Single Thread Usage:");
    let pool = ThreadLocalPool::<i32>::new(100).expect("Failed to create pool");

    println!("   Created thread-local pool");

    let mut handles = Vec::new();
    for i in 0..10 {
        handles.push(pool.allocate(i).expect("Allocation failed"));
    }

    println!("   Allocated 10 values");
    println!("   Pool: {}/{} used\n", pool.allocated(), pool.capacity());

    // Example 2: Each thread gets its own pool
    println!("2. Multiple Threads (Each with Own Pool):");

    let mut thread_handles = vec![];

    for thread_id in 0..4 {
        thread_handles.push(thread::spawn(move || {
            // Each thread creates its own pool
            let pool = ThreadLocalPool::<String>::new(50).expect("Failed to create pool");

            println!("   Thread {} created pool", thread_id);

            let mut items = Vec::new();
            for i in 0..20 {
                let s = format!("Thread-{}-Item-{}", thread_id, i);
                items.push(pool.allocate(s).expect("Allocation failed"));
            }

            println!("   Thread {} allocated {} items", thread_id, items.len());
            println!(
                "   Thread {} pool: {}/{} used",
                thread_id,
                pool.allocated(),
                pool.capacity()
            );

            // Process items
            let mut sum_len = 0;
            for item in &items {
                sum_len += item.len();
            }

            println!("   Thread {} total string length: {}", thread_id, sum_len);

            sum_len
        }));
    }

    println!("\n   Waiting for threads to complete...");
    for handle in thread_handles {
        let result = handle.join().expect("Thread panicked");
        println!("   Thread completed with result: {}", result);
    }

    // Example 3: Performance demonstration
    println!("\n3. Performance Test:");

    let iterations = 10000;

    let pool = ThreadLocalPool::<u64>::new(1000).expect("Failed to create pool");

    let start = std::time::Instant::now();

    for i in 0..iterations {
        let handle = pool.allocate(i).expect("Allocation failed");
        // Immediately drop to simulate high-churn scenario
        drop(handle);
    }

    let elapsed = start.elapsed();

    println!("   Completed {} allocations/deallocations", iterations);
    println!("   Time: {:?}", elapsed);
    println!(
        "   Average: {:.2}ns per operation",
        elapsed.as_nanos() as f64 / iterations as f64
    );

    // Example 4: Demonstrating the !Sync property
    println!("\n4. Thread Safety Note:");
    println!("   ThreadLocalPool is Send but not Sync");
    println!("   This means:");
    println!("   - ✓ Can be moved to another thread");
    println!("   - ✗ Cannot be shared between threads (no Arc<ThreadLocalPool>)");
    println!("   - This design prevents synchronization overhead");

    // Demonstrate moving pool to another thread
    let pool = ThreadLocalPool::<i32>::new(10).expect("Failed to create pool");
    let handle = thread::spawn(move || {
        // Pool was moved here
        let allocated = pool.allocate(42).expect("Allocation failed");
        *allocated // Return the value, not the handle
    });

    let value = handle.join().expect("Thread panicked");
    println!("\n   Pool moved to thread and allocated: {}", value);

    println!("\n=== Example Complete ===");
}
