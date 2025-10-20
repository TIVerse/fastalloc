//! Async runtime integration example using tokio.

use fastalloc::{FixedPool, ThreadSafePool, PoolConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("=== Async Usage Example ===\n");
    
    // Example 1: Thread-safe pool with async tasks
    println!("1. ThreadSafePool with Async Tasks:");
    
    let config = PoolConfig::builder()
        .capacity(100)
        .build()
        .unwrap();
    
    let pool = Arc::new(ThreadSafePool::with_config(config).unwrap());
    
    println!("   Created shared pool");
    
    let mut tasks = Vec::new();
    
    for task_id in 0..10 {
        let pool_clone = Arc::clone(&pool);
        
        let task = tokio::spawn(async move {
            // Allocate from pool, use it, then drop before await
            let value = {
                let mut handle = pool_clone.allocate(task_id * 10).expect("Allocation failed");
                
                println!("   Task {} allocated value: {}", task_id, *handle);
                
                // Modify value
                *handle += 5;
                
                println!("   Task {} modified value: {}", task_id, *handle);
                
                *handle
            }; // Handle dropped here, before await
            
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            value
        });
        
        tasks.push(task);
    }
    
    println!("\n   Waiting for tasks to complete...\n");
    
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.expect("Task failed"));
    }
    
    println!("   All tasks completed");
    println!("   Results: {:?}\n", results);
    
    // Example 2: Async request handling simulation
    println!("2. Simulated Async Request Handler:");
    
    #[derive(Clone)]
    struct Request {
        id: u64,
        path: String,
        data: Vec<u8>,
    }
    
    impl fastalloc::Poolable for Request {}
    
    let request_pool = Arc::new(ThreadSafePool::<Request>::new(50).unwrap());
    
    async fn handle_request(pool: Arc<ThreadSafePool<Request>>, id: u64) -> String {
        // Allocate request object from pool, process it, then drop before await
        let result = {
            let mut request = pool.allocate(Request {
                id,
                path: format!("/api/v1/data/{}", id),
                data: vec![0; 1024],
            }).expect("Failed to allocate request");
            
            request.data[0] = 42;
            
            format!("Processed request {} on {}", request.id, request.path)
        }; // Handle dropped here
        
        // Simulate async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        
        result
    }
    
    let mut handlers = Vec::new();
    
    for i in 0..20 {
        let pool_clone = Arc::clone(&request_pool);
        handlers.push(tokio::spawn(handle_request(pool_clone, i)));
    }
    
    println!("   Spawned 20 async request handlers");
    
    for handler in handlers {
        let result = handler.await.expect("Handler failed");
        println!("   {}", result);
    }
    
    println!("\n   Final pool state:");
    println!("   Capacity: {}", request_pool.capacity());
    println!("   Available: {}", request_pool.available());
    
    // Example 3: Concurrent stream processing
    println!("\n3. Concurrent Stream Processing:");
    
    let stream_pool = Arc::new(ThreadSafePool::<Vec<f64>>::new(20).unwrap());
    
    async fn process_chunk(pool: Arc<ThreadSafePool<Vec<f64>>>, chunk_id: usize) -> f64 {
        // Allocate, process, and extract result before awaiting
        let sum = {
            let mut chunk = pool.allocate(vec![0.0; 100]).expect("Allocation failed");
            
            // Simulate data processing
            for i in 0..100 {
                chunk[i] = (chunk_id * 100 + i) as f64;
            }
            
            chunk.iter().sum()
        }; // Handle dropped here
        
        tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
        
        sum
    }
    
    let mut chunk_tasks = Vec::new();
    
    for chunk_id in 0..10 {
        let pool_clone = Arc::clone(&stream_pool);
        chunk_tasks.push(tokio::spawn(process_chunk(pool_clone, chunk_id)));
    }
    
    let mut total_sum = 0.0;
    for task in chunk_tasks {
        total_sum += task.await.expect("Chunk processing failed");
    }
    
    println!("   Processed 10 data chunks");
    println!("   Total sum: {}", total_sum);
    
    // Example 4: Select! with multiple pool operations
    println!("\n4. Using select! with Pool Operations:");
    
    let pool_a = Arc::new(ThreadSafePool::<String>::new(10).unwrap());
    let pool_b = Arc::new(ThreadSafePool::<i32>::new(10).unwrap());
    
    let task_a = {
        let pool = Arc::clone(&pool_a);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            let handle = pool.allocate(String::from("Task A result")).unwrap();
            handle.clone() // Clone the value out
        })
    };
    
    let task_b = {
        let pool = Arc::clone(&pool_b);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let handle = pool.allocate(42).unwrap();
            *handle // Return the value, not the handle
        })
    };
    
    tokio::select! {
        result = task_a => {
            let value = result.unwrap();
            println!("   Task A completed first: {}", value);
        }
        result = task_b => {
            let value = result.unwrap();
            println!("   Task B completed first: {}", value);
        }
    }
    
    println!("\n=== Example Complete ===");
}
