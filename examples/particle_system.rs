//! Particle system example showing high-rate allocations.

use fastalloc::FixedPool;

#[derive(Debug, Clone, Copy)]
struct Particle {
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
    lifetime: f32,
    color: [u8; 4],
}

impl fastalloc::Poolable for Particle {}

fn main() {
    println!("=== Particle System Example ===\n");

    // Create large pool for particles
    let pool = FixedPool::<Particle>::new(5000).expect("Failed to create particle pool");

    println!("Particle pool created with capacity: {}\n", pool.capacity());

    let mut particles = Vec::new();

    // Burst emission
    println!("--- Burst Emission ---");
    let burst_count = 1000;
    for i in 0..burst_count {
        let angle = (i as f32 / burst_count as f32) * 2.0 * std::f32::consts::PI;
        let speed = 10.0;

        let particle = Particle {
            position: (0.0, 0.0, 0.0),
            velocity: (angle.cos() * speed, angle.sin() * speed, 0.0),
            lifetime: 2.0,
            color: [255, 200, 100, 255],
        };

        if let Ok(handle) = pool.allocate(particle) {
            particles.push(handle);
        }
    }

    println!("Emitted {} particles", particles.len());
    println!("Pool available: {}\n", pool.available());

    // Simulate frames
    println!("--- Simulating Frames ---");
    let dt = 0.016; // 60 FPS

    for frame in 0..120 {
        // Update particles
        for particle in particles.iter_mut() {
            particle.position.0 += particle.velocity.0 * dt;
            particle.position.1 += particle.velocity.1 * dt;
            particle.position.2 += particle.velocity.2 * dt;
            particle.lifetime -= dt;
        }

        // Remove dead particles
        let initial_count = particles.len();
        particles.retain(|p| p.lifetime > 0.0);
        let removed = initial_count - particles.len();

        // Continuous emission (less frequent)
        if frame % 10 == 0 {
            for i in 0..50 {
                let angle = (i as f32 / 50.0) * 2.0 * std::f32::consts::PI;
                let speed = 5.0;

                let particle = Particle {
                    position: (0.0, 0.0, 0.0),
                    velocity: (angle.cos() * speed, angle.sin() * speed, 2.0),
                    lifetime: 1.5,
                    color: [100, 150, 255, 255],
                };

                if let Ok(handle) = pool.allocate(particle) {
                    particles.push(handle);
                }
            }
        }

        if frame % 30 == 0 {
            println!(
                "Frame {}: {} active particles, {} removed, available: {}",
                frame,
                particles.len(),
                removed,
                pool.available()
            );
        }
    }

    println!("\n--- Final State ---");
    println!("Active particles: {}", particles.len());
    println!("Pool available: {}", pool.available());
    println!(
        "Pool utilization: {:.1}%",
        (pool.allocated() as f32 / pool.capacity() as f32) * 100.0
    );

    println!("\n=== Example Complete ===");
}
