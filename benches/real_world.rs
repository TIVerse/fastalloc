use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fastalloc::{FixedPool, PoolConfig};

// Simulated game entity
#[derive(Clone)]
struct GameEntity {
    id: u64,
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
    health: i32,
    active: bool,
}
impl fastalloc::Poolable for GameEntity {}

fn bench_game_entity_spawning(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_entity_spawning");

    group.bench_function("pool", |b| {
        let pool = FixedPool::<GameEntity>::new(1000).unwrap();

        b.iter(|| {
            let mut entities = Vec::new();

            // Spawn 100 entities
            for i in 0..100 {
                let entity = GameEntity {
                    id: i,
                    position: (0.0, 0.0, 0.0),
                    velocity: (1.0, 0.0, 0.0),
                    health: 100,
                    active: true,
                };
                entities.push(pool.allocate(entity).unwrap());
            }

            // Simulate some entities dying (deallocation)
            entities.drain(0..20);

            // Spawn more entities (reusing freed slots)
            for i in 100..120 {
                let entity = GameEntity {
                    id: i,
                    position: (0.0, 0.0, 0.0),
                    velocity: (1.0, 0.0, 0.0),
                    health: 100,
                    active: true,
                };
                entities.push(pool.allocate(entity).unwrap());
            }

            black_box(&entities);
        });
    });

    group.bench_function("box", |b| {
        b.iter(|| {
            let mut entities = Vec::new();

            for i in 0..100 {
                let entity = GameEntity {
                    id: i,
                    position: (0.0, 0.0, 0.0),
                    velocity: (1.0, 0.0, 0.0),
                    health: 100,
                    active: true,
                };
                entities.push(Box::new(entity));
            }

            entities.drain(0..20);

            for i in 100..120 {
                let entity = GameEntity {
                    id: i,
                    position: (0.0, 0.0, 0.0),
                    velocity: (1.0, 0.0, 0.0),
                    health: 100,
                    active: true,
                };
                entities.push(Box::new(entity));
            }

            black_box(&entities);
        });
    });

    group.finish();
}

// Simulated connection object
#[derive(Clone)]
struct Connection {
    id: u64,
    addr: [u8; 4],
    port: u16,
    buffer: Vec<u8>,
}
impl fastalloc::Poolable for Connection {}

fn bench_server_connections(c: &mut Criterion) {
    let mut group = c.benchmark_group("server_connections");

    group.bench_function("pool", |b| {
        let pool = FixedPool::<Connection>::new(500).unwrap();

        b.iter(|| {
            let mut connections = Vec::new();

            // Simulate incoming connections
            for i in 0..100 {
                let conn = Connection {
                    id: i,
                    addr: [127, 0, 0, 1],
                    port: 8080 + (i % 100) as u16,
                    buffer: Vec::with_capacity(1024),
                };
                connections.push(pool.allocate(conn).unwrap());
            }

            // Simulate some connections closing
            connections.drain(0..30);

            // New connections reuse slots
            for i in 100..130 {
                let conn = Connection {
                    id: i,
                    addr: [127, 0, 0, 1],
                    port: 8080 + (i % 100) as u16,
                    buffer: Vec::with_capacity(1024),
                };
                connections.push(pool.allocate(conn).unwrap());
            }

            black_box(&connections);
        });
    });

    group.finish();
}

// Simulated particle for particle system
#[derive(Clone, Copy)]
struct Particle {
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
    lifetime: f32,
    color: [u8; 4],
}
impl fastalloc::Poolable for Particle {}

fn bench_particle_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("particle_system");

    group.bench_function("pool_high_churn", |b| {
        let pool = FixedPool::<Particle>::new(2000).unwrap();

        b.iter(|| {
            let mut particles = Vec::new();

            // Burst emission
            for _ in 0..500 {
                let particle = Particle {
                    position: (0.0, 0.0, 0.0),
                    velocity: (1.0, 2.0, 0.5),
                    lifetime: 1.0,
                    color: [255, 255, 255, 255],
                };
                particles.push(pool.allocate(particle).unwrap());
            }

            // Simulate particle deaths and new emissions over time
            for _frame in 0..10 {
                // Remove dead particles
                particles.drain(0..particles.len().min(50));

                // Emit new particles
                for _ in 0..50 {
                    let particle = Particle {
                        position: (0.0, 0.0, 0.0),
                        velocity: (1.0, 2.0, 0.5),
                        lifetime: 1.0,
                        color: [255, 255, 255, 255],
                    };
                    if let Ok(p) = pool.allocate(particle) {
                        particles.push(p);
                    }
                }
            }

            black_box(&particles);
        });
    });

    group.finish();
}

// Data processing pipeline simulation
fn bench_data_pipeline(c: &mut Criterion) {
    #[derive(Clone)]
    struct DataChunk {
        id: u64,
        data: Vec<f64>,
        processed: bool,
    }
    impl fastalloc::Poolable for DataChunk {}

    let mut group = c.benchmark_group("data_pipeline");

    group.bench_function("pool", |b| {
        let pool = FixedPool::<DataChunk>::new(200).unwrap();

        b.iter(|| {
            let mut pipeline = Vec::new();

            // Stage 1: Create data chunks
            for i in 0..50 {
                let chunk = DataChunk {
                    id: i,
                    data: vec![0.0; 100],
                    processed: false,
                };
                pipeline.push(pool.allocate(chunk).unwrap());
            }

            // Stage 2: Process (simulate)
            for chunk in &mut pipeline {
                chunk.processed = true;
            }

            // Stage 3: Release processed chunks and add new ones
            pipeline.drain(0..25);

            for i in 50..75 {
                let chunk = DataChunk {
                    id: i,
                    data: vec![0.0; 100],
                    processed: false,
                };
                pipeline.push(pool.allocate(chunk).unwrap());
            }

            black_box(&pipeline);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_game_entity_spawning,
    bench_server_connections,
    bench_particle_system,
    bench_data_pipeline
);
criterion_main!(benches);
