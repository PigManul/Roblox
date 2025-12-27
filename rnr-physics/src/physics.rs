use rapier3d::prelude::*;
use glam::Vec3;
use std::collections::HashMap;

/// Physics world using Rapier3D (replaces Bullet)
pub struct PhysicsWorld {
    /// Rapier physics pipeline
    pub pipeline: PhysicsPipeline,
    /// Gravity vector
    pub gravity: Vec3,
    /// Integration parameters
    pub integration_parameters: IntegrationParameters,
    /// Island manager
    pub islands: IslandManager,
    /// Broad phase collision detection
    pub broad_phase: BroadPhase,
    /// Narrow phase collision detection
    pub narrow_phase: NarrowPhase,
    /// Rigid body set
    pub rigid_bodies: RigidBodySet,
    /// Collider set
    pub colliders: ColliderSet,
    /// Impulse joint set
    pub impulse_joints: ImpulseJointSet,
    /// Multibody joint set
    pub multibody_joints: MultibodyJointSet,
    /// CCD solver
    pub ccd_solver: CCDSolver,
    /// Collision event handler
    pub event_handler: (),
    /// Query pipeline for raycasting, etc.
    pub query_pipeline: QueryPipeline,
    /// Physics hooks
    pub hooks: (),
}

impl PhysicsWorld {
    /// Create a new physics world
    pub fn new() -> Self {
        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0), // Standard gravity
            integration_parameters: IntegrationParameters::default(),
            islands: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            event_handler: (),
            query_pipeline: QueryPipeline::new(),
            hooks: (),
        }
    }

    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;

        self.pipeline.step(
            &vector![self.gravity.x, self.gravity.y, self.gravity.z],
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.hooks,
            &self.event_handler,
        );
    }

    /// Set gravity
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    /// Get gravity
    pub fn get_gravity(&self) -> Vec3 {
        self.gravity
    }

    /// Add a rigid body
    pub fn add_rigid_body(&mut self, rigid_body: RigidBody) -> RigidBodyHandle {
        self.rigid_bodies.insert(rigid_body)
    }

    /// Remove a rigid body
    pub fn remove_rigid_body(&mut self, handle: RigidBodyHandle) -> Option<RigidBody> {
        self.rigid_bodies.remove(handle, &mut self.islands, &mut self.colliders, &mut self.impulse_joints, &mut self.multibody_joints, true)
    }

    /// Get a rigid body
    pub fn get_rigid_body(&self, handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.rigid_bodies.get(handle)
    }

    /// Get a mutable rigid body
    pub fn get_rigid_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_bodies.get_mut(handle)
    }

    /// Add a collider
    pub fn add_collider(&mut self, collider: Collider) -> ColliderHandle {
        self.colliders.insert(collider)
    }

    /// Remove a collider
    pub fn remove_collider(&mut self, handle: ColliderHandle) -> Option<Collider> {
        self.colliders.remove(handle, &mut self.islands, &mut self.rigid_bodies, false)
    }

    /// Cast a ray and get the first hit
    pub fn cast_ray(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<(ColliderHandle, f32)> {
        let ray = Ray::new(nalgebra::Point3::new(origin.x, origin.y, origin.z), nalgebra::Vector3::new(direction.x, direction.y, direction.z));
        let filter = QueryFilter::default();

        self.query_pipeline.cast_ray(&self.rigid_bodies, &self.colliders, &ray, max_distance, true, filter)
    }

    /// Cast a ray and get all hits
    pub fn cast_ray_all(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Vec<(ColliderHandle, RayIntersection)> {
        let ray = Ray::new(nalgebra::Point3::new(origin.x, origin.y, origin.z), nalgebra::Vector3::new(direction.x, direction.y, direction.z));
        let filter = QueryFilter::default();
        let mut hits = Vec::new();

        self.query_pipeline.intersections_with_ray(&self.rigid_bodies, &self.colliders, &ray, max_distance, true, filter, |handle, intersection| {
            hits.push((handle, intersection));
            true // Continue searching
        });

        hits
    }

    /// Check if a point is inside any collider
    pub fn point_projection(&self, point: Vec3) -> Option<(ColliderHandle, PointProjection)> {
        let filter = QueryFilter::default();
        self.query_pipeline.project_point(&self.rigid_bodies, &self.colliders, &nalgebra::Point3::new(point.x, point.y, point.z), true, filter)
    }

    /// Create a box collider
    pub fn create_box_collider(half_extents: Vec3) -> Collider {
        ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z).build()
    }

    /// Create a sphere collider
    pub fn create_sphere_collider(radius: f32) -> Collider {
        ColliderBuilder::ball(radius).build()
    }

    /// Create a capsule collider
    pub fn create_capsule_collider(half_height: f32, radius: f32) -> Collider {
        ColliderBuilder::capsule_y(half_height, radius).build()
    }

    /// Create a static rigid body
    pub fn create_static_body() -> RigidBody {
        RigidBodyBuilder::fixed().build()
    }

    /// Create a dynamic rigid body
    pub fn create_dynamic_body() -> RigidBody {
        RigidBodyBuilder::dynamic().build()
    }

    /// Create a kinematic rigid body
    pub fn create_kinematic_body() -> RigidBody {
        RigidBodyBuilder::kinematic_position_based().build()
    }
}

/// Computational Physics Engine (ComPlicitNgine) - handles complex physics calculations
pub struct ComPlicitNgine {
    /// Last physics delta time
    pub last_physics_delta: f32,
    /// Whether the engine is enabled
    pub enabled: bool,
    /// Custom physics calculations storage
    pub calculations: HashMap<String, Box<dyn Fn(f32) + Send + Sync>>,
}

impl ComPlicitNgine {
    /// Create a new computational physics engine
    pub fn new() -> Self {
        Self {
            last_physics_delta: 0.0,
            enabled: true,
            calculations: HashMap::new(),
        }
    }

    /// Step the computational physics engine
    pub fn step(&mut self, delta_time: f32) {
        self.last_physics_delta = delta_time;

        if !self.enabled {
            return;
        }

        // Run all registered calculations
        for calculation in self.calculations.values() {
            calculation(delta_time);
        }
    }

    /// Get the last physics delta time
    pub fn get_last_physics_delta(&self) -> f32 {
        self.last_physics_delta
    }

    /// Set whether the engine is enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Register a custom physics calculation
    pub fn register_calculation<F>(&mut self, name: &str, calculation: F)
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.calculations.insert(name.to_string(), Box::new(calculation));
    }

    /// Unregister a physics calculation
    pub fn unregister_calculation(&mut self, name: &str) {
        self.calculations.remove(name);
    }
}

impl Default for ComPlicitNgine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        let world = PhysicsWorld::new();

        assert_eq!(world.get_gravity(), Vec3::new(0.0, -9.81, 0.0));
        assert!(world.rigid_bodies.is_empty());
        assert!(world.colliders.is_empty());
    }

    #[test]
    fn test_physics_world_gravity() {
        let mut world = PhysicsWorld::new();

        world.set_gravity(Vec3::new(0.0, -20.0, 0.0));
        assert_eq!(world.get_gravity(), Vec3::new(0.0, -20.0, 0.0));
    }

    #[test]
    fn test_collider_creation() {
        let box_collider = PhysicsWorld::create_box_collider(Vec3::new(1.0, 2.0, 3.0));
        assert!(box_collider.shape().is_cuboid());

        let sphere_collider = PhysicsWorld::create_sphere_collider(5.0);
        assert!(sphere_collider.shape().is_ball());

        let capsule_collider = PhysicsWorld::create_capsule_collider(1.0, 0.5);
        assert!(capsule_collider.shape().is_capsule());
    }

    #[test]
    fn test_rigid_body_creation() {
        let static_body = PhysicsWorld::create_static_body();
        assert_eq!(static_body.body_type(), RigidBodyType::Fixed);

        let dynamic_body = PhysicsWorld::create_dynamic_body();
        assert_eq!(dynamic_body.body_type(), RigidBodyType::Dynamic);

        let kinematic_body = PhysicsWorld::create_kinematic_body();
        assert_eq!(kinematic_body.body_type(), RigidBodyType::KinematicPositionBased);
    }

    #[test]
    fn test_com_plicit_ngine() {
        let mut engine = ComPlicitNgine::new();

        assert!(engine.enabled);
        assert_eq!(engine.get_last_physics_delta(), 0.0);

        engine.step(0.016);
        assert_eq!(engine.get_last_physics_delta(), 0.016);

        engine.set_enabled(false);
        engine.step(0.032);
        assert_eq!(engine.get_last_physics_delta(), 0.032); // Should still update delta
    }

    #[test]
    fn test_custom_calculations() {
        let mut engine = ComPlicitNgine::new();

        let mut counter = std::sync::Mutex::new(0);
        engine.register_calculation("test_calc", move |_dt| {
            *counter.lock().unwrap() += 1;
        });

        engine.step(0.016);
        assert_eq!(*counter.lock().unwrap(), 1);

        engine.step(0.016);
        assert_eq!(*counter.lock().unwrap(), 2);

        engine.unregister_calculation("test_calc");
        engine.step(0.016);
        assert_eq!(*counter.lock().unwrap(), 2); // Should not increment
    }
}
