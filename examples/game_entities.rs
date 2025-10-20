//! Game entity pooling example showing spawn/despawn cycles.

use fastalloc::{FixedPool, OwnedHandle, Poolable};
use std::collections::HashMap;

#[derive(Debug)]
struct GameEntity {
    id: u64,
    position: (f32, f32),
    velocity: (f32, f32),
    health: i32,
    active: bool,
    entity_type: EntityType,
}

#[derive(Debug, Clone, Copy)]
enum EntityType {
    Player,
    Enemy,
    Projectile,
}

impl Poolable for GameEntity {
    fn on_acquire(&mut self) {
        // Reset entity state when acquired from pool
        self.position = (0.0, 0.0);
        self.velocity = (0.0, 0.0);
        self.health = 100;
        self.active = true;
        println!("  [Pool] Entity {} acquired and reset", self.id);
    }
    
    fn on_release(&mut self) {
        // Cleanup before returning to pool
        self.active = false;
        println!("  [Pool] Entity {} released", self.id);
    }
}

fn main() {
    println!("=== Game Entity Pooling Example ===\n");
    
    // Create a pool for game entities
    let pool = FixedPool::<GameEntity>::new(50)
        .expect("Failed to create entity pool");
    
    println!("Entity pool created with capacity: {}\n", pool.capacity());
    
    let mut next_id = 0u64;
    let mut active_entities = HashMap::new();
    
    // Helper function to spawn entity
    fn spawn_entity<'a>(
        pool: &'a FixedPool<GameEntity>, 
        entities: &mut HashMap<usize, OwnedHandle<'a, GameEntity>>, 
        id: &mut u64,
        entity_type: EntityType, 
        pos: (f32, f32)
    ) {
        let mut entity = GameEntity {
            id: *id,
            position: pos,
            velocity: (0.0, 0.0),
            health: 100,
            active: true,
            entity_type,
        };
        
        *id += 1;
        
        // Set type-specific properties
        match entity_type {
            EntityType::Player => {
                entity.health = 200;
                entity.velocity = (5.0, 0.0);
            }
            EntityType::Enemy => {
                entity.health = 50;
                entity.velocity = (-3.0, 0.0);
            }
            EntityType::Projectile => {
                entity.health = 1;
                entity.velocity = (10.0, 0.0);
            }
        }
        
        match pool.allocate(entity) {
            Ok(handle) => {
                let idx = handle.index();
                println!("Spawned {:?} at position {:?}", entity_type, pos);
                entities.insert(idx, handle);
            }
            Err(e) => {
                println!("Failed to spawn entity: {}", e);
            }
        }
    }
    
    // Spawn player
    println!("--- Spawning Player ---");
    spawn_entity(&pool, &mut active_entities, &mut next_id, EntityType::Player, (0.0, 0.0));
    
    println!("\n--- Spawning Enemies ---");
    for i in 0..5 {
        spawn_entity(&pool, &mut active_entities, &mut next_id, 
                    EntityType::Enemy, (100.0 + i as f32 * 20.0, 50.0));
    }
    
    println!("\nActive entities: {}", active_entities.len());
    println!("Pool available: {}\n", pool.available());
    
    // Simulate game loop frames
    println!("--- Simulating Game Loop ---");
    
    for frame in 0..3 {
        println!("\nFrame {}", frame);
        
        // Spawn some projectiles
        if frame % 2 == 0 {
            println!("  Player shoots:");
            for i in 0..3 {
                spawn_entity(&pool, &mut active_entities, &mut next_id,
                            EntityType::Projectile, (10.0, i as f32 * 5.0));
            }
        }
        
        // Update entities (simplified)
        for (_idx, entity) in active_entities.iter_mut() {
            entity.position.0 += entity.velocity.0 * 0.016; // 60 FPS
            entity.position.1 += entity.velocity.1 * 0.016;
        }
        
        // Remove entities that are out of bounds or dead
        let mut to_remove = Vec::new();
        for (idx, entity) in active_entities.iter() {
            if entity.position.0 > 200.0 || entity.position.0 < -200.0 || entity.health <= 0 {
                println!("  Despawning {:?} at {:?}", entity.entity_type, entity.position);
                to_remove.push(*idx);
            }
        }
        
        // Remove dead/out-of-bounds entities (returns them to pool)
        for idx in to_remove {
            active_entities.remove(&idx);
        }
        
        println!("  Active entities: {}", active_entities.len());
        println!("  Pool available: {}", pool.available());
    }
    
    println!("\n--- Cleanup ---");
    active_entities.clear();
    println!("All entities despawned");
    println!("Pool available: {}/{}", pool.available(), pool.capacity());
    
    // Demonstrate that pool memory is reused
    println!("\n--- Testing Reuse ---");
    println!("Spawning new entities...");
    for i in 0..10 {
        spawn_entity(&pool, &mut active_entities, &mut next_id,
                    EntityType::Enemy, (i as f32 * 10.0, 0.0));
    }
    
    println!("\nNew entities spawned: {}", active_entities.len());
    println!("Total entities ever created: {}", next_id);
    println!("Pool efficiently reused {} slots", 
             next_id as usize - active_entities.len());
    
    println!("\n=== Example Complete ===");
}
