use glam::{Vec3, Quat};
use std::rc::Rc;
use std::cell::RefCell;
use rnr_core::instance::Instance;

/// Humanoid character controller
pub struct Humanoid {
    instance: Rc<RefCell<Instance>>,
    /// Character position
    pub position: Vec3,
    /// Character rotation
    pub rotation: Quat,
    /// Movement velocity
    pub velocity: Vec3,
    /// Whether the humanoid is on the ground
    pub on_ground: bool,
    /// Jump power
    pub jump_power: f32,
    /// Walk speed
    pub walk_speed: f32,
    /// Run speed
    pub run_speed: f32,
    /// Current speed multiplier
    pub speed_multiplier: f32,
    /// Health points
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
}

impl Humanoid {
    /// Create a new humanoid
    pub fn new() -> Rc<RefCell<Self>> {
        let instance = Instance::new();
        instance.borrow_mut().set_class_name("Humanoid");

        Rc::new(RefCell::new(Self {
            instance,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            on_ground: true,
            jump_power: 50.0,
            walk_speed: 16.0,
            run_speed: 32.0,
            speed_multiplier: 1.0,
            health: 100.0,
            max_health: 100.0,
        }))
    }

    /// Get the underlying instance
    pub fn instance(&self) -> &Rc<RefCell<Instance>> {
        &self.instance
    }

    /// Move the humanoid in a direction
    pub fn move_direction(&mut self, direction: Vec3, delta_time: f32) {
        if direction != Vec3::ZERO {
            let normalized_dir = direction.normalize();
            let speed = self.walk_speed * self.speed_multiplier;
            self.velocity = normalized_dir * speed;

            // Update position
            self.position += self.velocity * delta_time;

            // Update rotation to face movement direction
            if normalized_dir != Vec3::ZERO {
                self.rotation = Quat::from_rotation_arc(Vec3::Z, normalized_dir);
            }
        } else {
            self.velocity = Vec3::ZERO;
        }
    }

    /// Make the humanoid jump
    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = self.jump_power;
            self.on_ground = false;
        }
    }

    /// Apply gravity and ground collision
    pub fn update_physics(&mut self, delta_time: f32, gravity: f32, ground_y: f32) {
        // Apply gravity
        self.velocity.y -= gravity * delta_time;

        // Update position
        self.position += self.velocity * delta_time;

        // Ground collision
        if self.position.y <= ground_y {
            self.position.y = ground_y;
            self.velocity.y = 0.0;
            self.on_ground = true;
        }
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }

    /// Heal the humanoid
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Check if the humanoid is alive
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Set walk speed
    pub fn set_walk_speed(&mut self, speed: f32) {
        self.walk_speed = speed;
    }

    /// Set run speed
    pub fn set_run_speed(&mut self, speed: f32) {
        self.run_speed = speed;
    }

    /// Set jump power
    pub fn set_jump_power(&mut self, power: f32) {
        self.jump_power = power;
    }

    /// Set speed multiplier (for buffs/debuffs)
    pub fn set_speed_multiplier(&mut self, multiplier: f32) {
        self.speed_multiplier = multiplier;
    }

    /// Get current speed (walk speed * multiplier)
    pub fn get_current_speed(&self) -> f32 {
        self.walk_speed * self.speed_multiplier
    }

    /// Get health percentage (0.0 to 1.0)
    pub fn get_health_percentage(&self) -> f32 {
        if self.max_health > 0.0 {
            self.health / self.max_health
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanoid_creation() {
        let humanoid = Humanoid::new();

        assert_eq!(humanoid.borrow().position, Vec3::ZERO);
        assert_eq!(humanoid.borrow().rotation, Quat::IDENTITY);
        assert_eq!(humanoid.borrow().health, 100.0);
        assert_eq!(humanoid.borrow().walk_speed, 16.0);
        assert!(humanoid.borrow().is_alive());
    }

    #[test]
    fn test_humanoid_movement() {
        let mut humanoid = Humanoid::new();
        let mut humanoid_ref = humanoid.borrow_mut();

        // Move forward
        humanoid_ref.move_direction(Vec3::new(0.0, 0.0, 1.0), 0.016);

        assert!(humanoid_ref.velocity.z > 0.0);
        assert!(humanoid_ref.position.z > 0.0);
    }

    #[test]
    fn test_humanoid_jump() {
        let mut humanoid = Humanoid::new();
        let mut humanoid_ref = humanoid.borrow_mut();

        humanoid_ref.jump();

        assert!(humanoid_ref.velocity.y > 0.0);
        assert!(!humanoid_ref.on_ground);

        // Test can't jump again while in air
        humanoid_ref.jump();
        assert_eq!(humanoid_ref.velocity.y, 50.0); // Should not change
    }

    #[test]
    fn test_humanoid_physics() {
        let mut humanoid = Humanoid::new();
        let mut humanoid_ref = humanoid.borrow_mut();

        // Start in air
        humanoid_ref.position.y = 10.0;
        humanoid_ref.on_ground = false;

        // Apply physics
        humanoid_ref.update_physics(0.016, 9.81, 0.0);

        assert!(humanoid_ref.position.y < 10.0); // Should fall
        assert!(humanoid_ref.velocity.y < 0.0); // Should have downward velocity
    }

    #[test]
    fn test_humanoid_health() {
        let mut humanoid = Humanoid::new();
        let mut humanoid_ref = humanoid.borrow_mut();

        humanoid_ref.take_damage(30.0);
        assert_eq!(humanoid_ref.health, 70.0);
        assert!(humanoid_ref.is_alive());

        humanoid_ref.take_damage(80.0);
        assert_eq!(humanoid_ref.health, 0.0);
        assert!(!humanoid_ref.is_alive());

        humanoid_ref.heal(50.0);
        assert_eq!(humanoid_ref.health, 50.0);
        assert!(humanoid_ref.is_alive());
    }

    #[test]
    fn test_humanoid_speed() {
        let mut humanoid = Humanoid::new();
        let mut humanoid_ref = humanoid.borrow_mut();

        assert_eq!(humanoid_ref.get_current_speed(), 16.0);

        humanoid_ref.set_speed_multiplier(2.0);
        assert_eq!(humanoid_ref.get_current_speed(), 32.0);

        humanoid_ref.set_walk_speed(20.0);
        assert_eq!(humanoid_ref.get_current_speed(), 40.0);
    }

    #[test]
    fn test_humanoid_health_percentage() {
        let mut humanoid = Humanoid::new();
        let mut humanoid_ref = humanoid.borrow_mut();

        assert_eq!(humanoid_ref.get_health_percentage(), 1.0);

        humanoid_ref.take_damage(50.0);
        assert_eq!(humanoid_ref.get_health_percentage(), 0.5);

        humanoid_ref.take_damage(60.0);
        assert_eq!(humanoid_ref.get_health_percentage(), 0.0);
    }
}
