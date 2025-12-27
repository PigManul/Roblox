use std::rc::Rc;
use std::cell::RefCell;
use rnr_datamodel::DataModel;

/// Configuration for creating a World instance
#[derive(Debug, Clone)]
pub struct WorldConfig {
    pub enable_rendering: bool,
    pub enable_networking: bool,
    pub enable_physics: bool,
    pub enable_input: bool,
    pub target_fps: u32,
    pub viewport_width: u32,
    pub viewport_height: u32,
}

/// Represents the game world containing all game state
pub struct World {
    config: WorldConfig,
    datamodel: Rc<RefCell<DataModel>>,
}

impl World {
    /// Create a new world with the given configuration
    pub fn new(config: WorldConfig) -> Self {
        let datamodel = DataModel::new();
        Self {
            config,
            datamodel,
        }
    }

    /// Initialize the world
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize services based on configuration
        println!("Initializing world with config: {:?}", self.config);
        Ok(())
    }

    /// Step the world forward by one frame
    pub async fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Update all systems
        Ok(())
    }

    /// Shutdown the world
    pub async fn shutdown(&mut self) {
        println!("Shutting down world...");
    }

    /// Get the data model
    pub fn datamodel(&self) -> &Rc<RefCell<DataModel>> {
        &self.datamodel
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_world_creation() {
        let config = WorldConfig {
            enable_rendering: true,
            enable_networking: true,
            enable_physics: true,
            enable_input: true,
            target_fps: 60,
            viewport_width: 800,
            viewport_height: 600,
        };

        let world = World::new(config);
        assert!(world.datamodel().borrow().services().is_empty());
    }
}
