use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use glam::Mat4;
use rnr_core::instance::Instance;
use rnr_datamodel::DataModel;
use rnr_rendering::{Renderer, Camera};

use crate::physics::{PhysicsWorld, ComPlicitNgine};

/// Loading states for the world
#[derive(Debug, Clone, PartialEq)]
pub enum WorldLoadState {
    LoadingDataModel,
    LoadingDataModelProperties,
    LoadingMakeJoints,
    Finished,
}

/// Interface for loading progress callbacks
pub trait LoadListener {
    fn update_world_load(&mut self);
}

/// The main World class - the core of the RNR engine
pub struct World {
    /// The data model containing all instances
    pub datamodel: Rc<RefCell<DataModel>>,
    /// The physics world (replaces Bullet)
    pub physics_world: PhysicsWorld,
    /// The computational physics engine
    pub com_plicit_ngine: ComPlicitNgine,
    /// The renderer
    pub renderer: Renderer,
    /// The active camera
    pub camera: Option<Camera>,
    /// Instance references for loading
    pub refs: HashMap<String, Rc<RefCell<Instance>>>,
    /// Undeserialized instances during loading
    pub undeserialized: Vec<WorldUndeserialized>,
    /// Current loading state
    pub load_state: WorldLoadState,
    /// Maximum load progress
    pub max_load_progress: i32,
    /// Current load progress
    pub load_progress: i32,
    /// Load listener callback
    pub load_listener: Option<Box<dyn LoadListener>>,
    /// Whether physics should be running
    pub run_physics: bool,
    /// Whether the scene has rendering capabilities
    pub scene_has_render: bool,
    /// Last physics delta time
    pub last_physics_delta: f32,
}

#[derive(Debug)]
pub struct WorldUndeserialized {
    pub instance: Rc<RefCell<Instance>>,
    pub parent: Option<Rc<RefCell<Instance>>>,
    pub xml_node: String, // Simplified, would be proper XML in real implementation
}

impl World {
    /// Create a new world
    pub fn new(has_render: bool) -> Self {
        let datamodel = DataModel::new();

        Self {
            datamodel: datamodel.clone(),
            physics_world: PhysicsWorld::new(),
            com_plicit_ngine: ComPlicitNgine::new(),
            renderer: Renderer::new(),
            camera: None,
            refs: HashMap::new(),
            undeserialized: Vec::new(),
            load_state: WorldLoadState::Finished,
            max_load_progress: 0,
            load_progress: 0,
            load_listener: None,
            run_physics: true,
            scene_has_render: has_render,
            last_physics_delta: 0.0,
        }
    }

    /// Set the active camera
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = Some(camera.clone());
        self.renderer.set_camera(camera);
    }

    /// Get the active camera
    pub fn get_camera(&self) -> Option<&Camera> {
        self.camera.as_ref()
    }

    /// Load a world from XML/path (simplified implementation)
    pub fn load(&mut self, _path: &str, load_listener: Option<Box<dyn LoadListener>>) {
        self.load_listener = load_listener;
        self.load_state = WorldLoadState::LoadingDataModel;
        self.max_load_progress = 100;
        self.load_progress = 0;

        // TODO: Implement actual XML loading
        // For now, just simulate loading
        self.load_progress = 50;
        if let Some(ref mut listener) = self.load_listener {
            listener.update_world_load();
        }

        self.load_state = WorldLoadState::LoadingDataModelProperties;
        self.load_progress = 75;
        if let Some(ref mut listener) = self.load_listener {
            listener.update_world_load();
        }

        self.load_state = WorldLoadState::LoadingMakeJoints;
        self.load_progress = 90;
        if let Some(ref mut listener) = self.load_listener {
            listener.update_world_load();
        }

        self.load_state = WorldLoadState::Finished;
        self.load_progress = 100;
        if let Some(ref mut listener) = self.load_listener {
            listener.update_world_load();
        }
    }

    /// Pre-render update
    pub fn pre_render(&mut self, timestep: f32) {
        // Update physics
        if self.run_physics {
            self.step_physics(timestep);
        }

        // Update renderer
        if self.scene_has_render {
            // TODO: Update camera matrices, etc.
        }
    }

    /// Step physics simulation
    pub fn step_physics(&mut self, timestep: f32) {
        self.last_physics_delta = timestep;
        self.physics_world.step(timestep);
        self.com_plicit_ngine.step(timestep);
    }

    /// Main update loop
    pub fn update(&mut self) {
        // Update all instances in the datamodel
        // TODO: Implement instance updating

        // Update physics
        if self.run_physics {
            // TODO: Step physics with appropriate timestep
        }
    }

    /// Get a reference by name
    pub fn get_ref(&self, ref_name: &str) -> Option<&Rc<RefCell<Instance>>> {
        self.refs.get(ref_name)
    }

    /// Set whether physics should run
    pub fn set_run_physics(&mut self, run_physics: bool) {
        self.run_physics = run_physics;
    }

    /// Check if physics should be running
    pub fn should_run_physics(&self) -> bool {
        self.run_physics
    }

    /// Get the last physics delta time
    pub fn get_last_physics_delta(&self) -> f32 {
        self.last_physics_delta
    }

    /// Add an instance to be rendered
    pub fn draw_mesh(&mut self, mesh_name: &str, material_name: &str, transform: Mat4, color: glam::Vec4) {
        self.renderer.draw_mesh(mesh_name, material_name, transform, color);
    }

    /// Render the current frame
    pub fn render_frame(&mut self) -> Result<(), rnr_rendering::RenderError> {
        self.renderer.render_frame()
    }

    /// Get render statistics
    pub fn get_render_stats(&self) -> rnr_rendering::RenderStats {
        self.renderer.get_stats()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rnr_rendering::Camera;

    #[test]
    fn test_world_creation() {
        let world = World::new(true);

        assert!(world.scene_has_render);
        assert!(!world.run_physics); // Should be false by default in new()
        assert_eq!(world.load_state, WorldLoadState::Finished);
    }

    #[test]
    fn test_world_loading() {
        let mut world = World::new(true);

        // Create a simple mock load listener
        struct MockLoadListener {
            pub update_count: std::cell::RefCell<i32>,
        }

        impl LoadListener for MockLoadListener {
            fn update_world_load(&mut self) {
                *self.update_count.borrow_mut() += 1;
            }
        }

        let listener = MockLoadListener {
            update_count: std::cell::RefCell::new(0),
        };

        world.load("test_path", Some(Box::new(listener)));

        assert_eq!(world.load_state, WorldLoadState::Finished);
        assert_eq!(world.load_progress, 100);
    }

    #[test]
    fn test_camera_setup() {
        let mut world = World::new(true);
        let camera = Camera::new(glam::Vec3::new(0.0, 0.0, 5.0), glam::Vec3::ZERO);

        world.set_camera(camera);

        assert!(world.get_camera().is_some());
    }

    #[test]
    fn test_physics_control() {
        let mut world = World::new(true);

        world.set_run_physics(true);
        assert!(world.should_run_physics());

        world.set_run_physics(false);
        assert!(!world.should_run_physics());
    }

    #[test]
    fn test_render_commands() {
        let mut world = World::new(true);

        // Add a camera first
        let camera = Camera::new(glam::Vec3::new(0.0, 0.0, 5.0), glam::Vec3::ZERO);
        world.set_camera(camera);

        world.draw_mesh(
            "Cube",
            "InstancedMaterial",
            glam::Mat4::IDENTITY,
            glam::Vec4::new(1.0, 0.0, 0.0, 1.0)
        );

        // Should succeed with camera set
        assert!(world.render_frame().is_ok());
    }
}
