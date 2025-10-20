//! Server connection pooling example.

use fastalloc::FixedPool;
use std::time::SystemTime;

#[derive(Debug)]
struct Connection {
    id: u64,
    client_addr: String,
    connected_at: SystemTime,
    bytes_sent: usize,
    bytes_received: usize,
    active: bool,
}

impl fastalloc::Poolable for Connection {}

fn main() {
    println!("=== Server Connection Pooling Example ===\n");
    
    // Create a pool for connections (typical server might have 1000-10000)
    let pool = FixedPool::<Connection>::new(100)
        .expect("Failed to create connection pool");
    
    println!("Connection pool created with capacity: {}\n", pool.capacity());
    
    let mut active_connections = Vec::new();
    let mut next_id = 0u64;
    
    // Simulate incoming connections
    println!("--- Accepting Connections ---");
    for i in 0..10 {
        let conn = Connection {
            id: next_id,
            client_addr: format!("192.168.1.{}", 100 + i),
            connected_at: SystemTime::now(),
            bytes_sent: 0,
            bytes_received: 0,
            active: true,
        };
        
        match pool.allocate(conn) {
            Ok(handle) => {
                println!("Accepted connection {} from {}", handle.id, handle.client_addr);
                active_connections.push(handle);
                next_id += 1;
            }
            Err(e) => {
                println!("Failed to accept connection: {}", e);
            }
        }
    }
    
    println!("\nActive connections: {}", active_connections.len());
    println!("Pool available: {}\n", pool.available());
    
    // Simulate some data transfer
    println!("--- Processing Connections ---");
    for conn in active_connections.iter_mut() {
        conn.bytes_sent += 1024;
        conn.bytes_received += 512;
        println!("Connection {}: sent={}, received={}", 
                 conn.id, conn.bytes_sent, conn.bytes_received);
    }
    
    // Simulate some connections closing
    println!("\n--- Closing Some Connections ---");
    let closed_count = 3;
    for _ in 0..closed_count {
        if let Some(conn) = active_connections.pop() {
            println!("Closing connection {} from {}", conn.id, conn.client_addr);
        }
    }
    
    println!("\nActive connections: {}", active_connections.len());
    println!("Pool available: {} (connections returned to pool)\n", pool.available());
    
    // Accept new connections (reusing freed slots)
    println!("--- Accepting New Connections (Reusing Slots) ---");
    for i in 0..5 {
        let conn = Connection {
            id: next_id,
            client_addr: format!("10.0.0.{}", 50 + i),
            connected_at: SystemTime::now(),
            bytes_sent: 0,
            bytes_received: 0,
            active: true,
        };
        
        match pool.allocate(conn) {
            Ok(handle) => {
                println!("Accepted connection {} from {} (reused pool slot)", 
                         handle.id, handle.client_addr);
                active_connections.push(handle);
                next_id += 1;
            }
            Err(e) => {
                println!("Failed to accept connection: {}", e);
            }
        }
    }
    
    println!("\nFinal active connections: {}", active_connections.len());
    println!("Pool available: {}", pool.available());
    println!("Total connections ever created: {}", next_id);
    
    #[cfg(feature = "stats")]
    {
        println!("\n--- Pool Statistics ---");
        let stats = pool.statistics();
        println!("{}", stats);
    }
    
    println!("\n=== Example Complete ===");
}
