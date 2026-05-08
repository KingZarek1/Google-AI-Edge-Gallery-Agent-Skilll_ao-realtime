// HyperTech Inc. - 3D Physics Engine
// Written in Rust for high-performance bubble physics simulation
// Compiled to WebAssembly (WASM) for browser integration

use wasm_bindgen::prelude::*;
use std::f32::consts::PI;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub fn add(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn subtract(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn multiply(&self, scalar: f32) -> Vector3 {
        Vector3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }

    pub fn dot(&self, other: &Vector3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vector3) -> Vector3 {
        Vector3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vector3 {
        let mag = self.magnitude();
        if mag == 0.0 {
            Vector3::new(0.0, 0.0, 0.0)
        } else {
            Vector3 {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        }
    }

    pub fn distance_to(&self, other: &Vector3) -> f32 {
        self.subtract(other).magnitude()
    }
}

#[wasm_bindgen]
pub struct Bubble {
    pub position: Vector3,
    pub velocity: Vector3,
    pub acceleration: Vector3,
    pub radius: f32,
    pub mass: f32,
    pub rotation: Vector3,
    pub angular_velocity: Vector3,
}

impl Bubble {
    pub fn new(position: Vector3, radius: f32) -> Bubble {
        Bubble {
            position,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            radius,
            mass: radius * radius * radius, // Volume-based mass
            rotation: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(
                (rand() - 0.5) * 0.1,
                (rand() - 0.5) * 0.1,
                (rand() - 0.5) * 0.1,
            ),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Apply acceleration to velocity
        self.velocity = self.velocity.add(&self.acceleration.multiply(delta_time));

        // Apply gravity
        self.velocity.y -= 9.8 * delta_time * 0.01;

        // Update position
        self.position = self.position.add(&self.velocity.multiply(delta_time));

        // Update rotation
        self.rotation = self.rotation.add(&self.angular_velocity.multiply(delta_time));

        // Air resistance
        self.velocity = self.velocity.multiply(0.99);
        self.acceleration = Vector3::new(0.0, 0.0, 0.0);
    }

    pub fn apply_force(&mut self, force: &Vector3) {
        let acceleration = force.multiply(1.0 / self.mass);
        self.acceleration = self.acceleration.add(&acceleration);
    }

    pub fn handle_collision(&mut self, other: &mut Bubble) {
        let distance = self.position.distance_to(&other.position);
        let min_distance = self.radius + other.radius;

        if distance < min_distance && distance > 0.0 {
            // Normal vector
            let normal = self.position.subtract(&other.position).normalize();

            // Relative velocity
            let relative_velocity = self.velocity.subtract(&other.velocity);
            let velocity_along_normal = relative_velocity.dot(&normal);

            // Don't process if moving away
            if velocity_along_normal >= 0.0 {
                return;
            }

            // Coefficient of restitution (bounciness)
            let e = 0.8;

            // Impulse scalar
            let impulse_scalar = -(1.0 + e) * velocity_along_normal 
                / (1.0 / self.mass + 1.0 / other.mass);

            let impulse = normal.multiply(impulse_scalar);

            // Apply impulse
            self.velocity = self.velocity.add(&impulse.multiply(1.0 / self.mass));
            other.velocity = other.velocity.subtract(&impulse.multiply(1.0 / other.mass));

            // Separate overlapping bubbles
            let overlap = (min_distance - distance) * 0.5;
            self.position = self.position.add(&normal.multiply(overlap));
            other.position = other.position.subtract(&normal.multiply(overlap));
        }
    }

    pub fn handle_boundary(&mut self, boundary: f32) {
        let damping = 0.8;

        // X axis
        if self.position.x + self.radius > boundary {
            self.position.x = boundary - self.radius;
            self.velocity.x *= -damping;
        } else if self.position.x - self.radius < -boundary {
            self.position.x = -boundary + self.radius;
            self.velocity.x *= -damping;
        }

        // Y axis
        if self.position.y + self.radius > boundary {
            self.position.y = boundary - self.radius;
            self.velocity.y *= -damping;
        } else if self.position.y - self.radius < -boundary {
            self.position.y = -boundary + self.radius;
            self.velocity.y *= -damping;
        }

        // Z axis
        if self.position.z + self.radius > boundary {
            self.position.z = boundary - self.radius;
            self.velocity.z *= -damping;
        } else if self.position.z - self.radius < -boundary {
            self.position.z = -boundary + self.radius;
            self.velocity.z *= -damping;
        }
    }
}

#[wasm_bindgen]
pub struct PhysicsEngine {
    bubbles: Vec<Bubble>,
    gravity: Vector3,
    damping: f32,
    boundary: f32,
}

#[wasm_bindgen]
impl PhysicsEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(bubble_count: usize, boundary: f32) -> PhysicsEngine {
        let mut bubbles = Vec::new();

        for _ in 0..bubble_count {
            let position = Vector3::new(
                (rand() - 0.5) * boundary * 2.0,
                (rand() - 0.5) * boundary * 2.0,
                (rand() - 0.5) * boundary,
            );
            let radius = 8.0 + rand() * 8.0;
            bubbles.push(Bubble::new(position, radius));
        }

        PhysicsEngine {
            bubbles,
            gravity: Vector3::new(0.0, -9.8, 0.0),
            damping: 0.99,
            boundary,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update each bubble
        for bubble in &mut self.bubbles {
            bubble.apply_force(&self.gravity.multiply(bubble.mass));
            bubble.update(delta_time);
            bubble.handle_boundary(self.boundary);
        }

        // Check collisions between bubbles
        let len = self.bubbles.len();
        for i in 0..len {
            for j in (i + 1)..len {
                let (left, right) = self.bubbles.split_at_mut(j);
                left[i].handle_collision(&mut right[0]);
            }
        }
    }

    pub fn get_bubble_data(&self) -> Vec<f32> {
        let mut data = Vec::new();
        for bubble in &self.bubbles {
            data.push(bubble.position.x);
            data.push(bubble.position.y);
            data.push(bubble.position.z);
            data.push(bubble.velocity.x);
            data.push(bubble.velocity.y);
            data.push(bubble.velocity.z);
            data.push(bubble.rotation.x);
            data.push(bubble.rotation.y);
            data.push(bubble.rotation.z);
            data.push(bubble.radius);
        }
        data
    }

    pub fn apply_force_to_bubble(&mut self, index: usize, fx: f32, fy: f32, fz: f32) {
        if index < self.bubbles.len() {
            self.bubbles[index].apply_force(&Vector3::new(fx, fy, fz));
        }
    }

    pub fn set_bubble_velocity(&mut self, index: usize, vx: f32, vy: f32, vz: f32) {
        if index < self.bubbles.len() {
            self.bubbles[index].velocity = Vector3::new(vx, vy, vz);
        }
    }
}

// Pseudo-random number generator
fn rand() -> f32 {
    // Simple LCG for deterministic but varied output
    static mut SEED: u32 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        ((SEED / 65536) % 32768) as f32 / 32768.0
    }
}

// Export WASM initialization
#[wasm_bindgen(start)]
pub fn init() {
    // Initialize panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_operations() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);

        let sum = v1.add(&v2);
        assert_eq!(sum.x, 5.0);
        assert_eq!(sum.y, 7.0);
        assert_eq!(sum.z, 9.0);

        let dot = v1.dot(&v2);
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6

        let magnitude = v1.magnitude();
        assert!((magnitude - 3.74166).abs() < 0.001);
    }

    #[test]
    fn test_bubble_physics() {
        let mut bubble = Bubble::new(Vector3::new(0.0, 0.0, 0.0), 5.0);
        let force = Vector3::new(10.0, 0.0, 0.0);
        
        bubble.apply_force(&force);
        bubble.update(0.016); // ~60 FPS
        
        assert!(bubble.position.x > 0.0);
    }

    #[test]
    fn test_collision_detection() {
        let mut bubble1 = Bubble::new(Vector3::new(-5.0, 0.0, 0.0), 5.0);
        let mut bubble2 = Bubble::new(Vector3::new(5.0, 0.0, 0.0), 5.0);
        
        bubble1.velocity = Vector3::new(5.0, 0.0, 0.0);
        bubble2.velocity = Vector3::new(-5.0, 0.0, 0.0);
        
        bubble1.handle_collision(&mut bubble2);
        
        // After collision, velocities should reverse
        assert!(bubble1.velocity.x < 0.0);
        assert!(bubble2.velocity.x > 0.0);
    }
}
