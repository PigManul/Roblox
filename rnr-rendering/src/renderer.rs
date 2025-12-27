use crate::{material::*, shader::*, mesh::*, texture::*, camera::*};
use glam::{Mat4, Vec4};

/// Main renderer responsible for drawing 3D graphics
pub struct Renderer {
    pub material_manager: MaterialManager,
    pub shader_manager: ShaderManager,
    pub mesh_manager: MeshManager,
    pub texture_manager: TextureManager,
    pub camera: Option<Camera>,
    pub render_queue: Vec<RenderCommand>,
}

#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub mesh_name: String,
    pub material_name: String,
    pub transform: Mat4,
    pub color: Vec4,
}

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        let mut renderer = Self {
            material_manager: MaterialManager::new(),
            shader_manager: ShaderManager::new(),
            mesh_manager: MeshManager::new(),
            texture_manager: TextureManager::new(),
            camera: None,
            render_queue: Vec::new(),
        };

        // Initialize default resources
        renderer.initialize_defaults();

        renderer
    }

    /// Initialize default resources
    fn initialize_defaults(&mut self) {
        // Create default textures
        self.texture_manager.create_default_textures();

        // Create default meshes
        self.mesh_manager.create_default_meshes();

        // Create default shaders
        self.shader_manager.create_instanced_shader();

        // Create default materials
        self.material_manager.create_instanced_material();
        self.material_manager.create_instanced_material_transparent();
    }

    /// Set the active camera
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
    }

    /// Get the active camera
    pub fn get_camera(&self) -> Option<&Camera> {
        self.camera.as_ref()
    }

    /// Add a render command to the queue
    pub fn draw_mesh(&mut self, mesh_name: &str, material_name: &str, transform: Mat4, color: Vec4) {
        self.render_queue.push(RenderCommand {
            mesh_name: mesh_name.to_string(),
            material_name: material_name.to_string(),
            transform,
            color,
        });
    }

    /// Clear the render queue
    pub fn clear_queue(&mut self) {
        self.render_queue.clear();
    }

    /// Render all queued commands (this would be called by the actual rendering backend)
    pub fn render_frame(&mut self) -> Result<(), RenderError> {
        if self.camera.is_none() {
            return Err(RenderError::NoCamera);
        }

        let _camera = self.camera.as_ref().unwrap();
        let _view_proj_matrix = _camera.view_projection_matrix();

        // In a real implementation, this would:
        // 1. Sort render commands by material/shader
        // 2. Set up render state (shaders, uniforms, etc.)
        // 3. Draw each mesh with its material

        for command in &self.render_queue {
            // Validate resources exist
            if self.mesh_manager.get_mesh(&command.mesh_name).is_none() {
                eprintln!("Warning: Mesh '{}' not found", command.mesh_name);
                continue;
            }

            if self.material_manager.get_material(&command.material_name).is_none() {
                eprintln!("Warning: Material '{}' not found", command.material_name);
                continue;
            }

            // Here would be the actual drawing code with wgpu or similar
            // For now, we just validate the command
        }

        // Clear queue after rendering
        self.render_queue.clear();

        Ok(())
    }

    /// Get render statistics
    pub fn get_stats(&self) -> RenderStats {
        RenderStats {
            materials_count: self.material_manager.materials.len(),
            shaders_count: self.shader_manager.programs.len(),
            meshes_count: self.mesh_manager.meshes.len(),
            textures_count: self.texture_manager.textures.len(),
            queued_commands: self.render_queue.len(),
        }
    }
}

#[derive(Debug)]
pub struct RenderStats {
    pub materials_count: usize,
    pub shaders_count: usize,
    pub meshes_count: usize,
    pub textures_count: usize,
    pub queued_commands: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("No active camera set")]
    NoCamera,
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),
    #[error("Material not found: {0}")]
    MaterialNotFound(String),
    #[error("Mesh not found: {0}")]
    MeshNotFound(String),
}

/// Render pass for organizing rendering
#[derive(Debug, Clone)]
pub struct RenderPass {
    pub name: String,
    pub clear_color: Option<Vec4>,
    pub clear_depth: Option<f32>,
    pub commands: Vec<RenderCommand>,
}

impl RenderPass {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            clear_color: Some(Vec4::new(0.1, 0.1, 0.1, 1.0)), // Dark gray
            clear_depth: Some(1.0),
            commands: Vec::new(),
        }
    }

    pub fn with_clear_color(mut self, color: Vec4) -> Self {
        self.clear_color = Some(color);
        self
    }

    pub fn without_clear(mut self) -> Self {
        self.clear_color = None;
        self.clear_depth = None;
        self
    }

    pub fn add_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }
}

/// Instanced rendering batch for efficient rendering of multiple instances
#[derive(Debug)]
pub struct InstanceBatch {
    pub mesh_name: String,
    pub material_name: String,
    pub instances: Vec<InstanceData>,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceData {
    pub transform: Mat4,
    pub color: Vec4,
}

unsafe impl bytemuck::Pod for InstanceData {}
unsafe impl bytemuck::Zeroable for InstanceData {}

impl InstanceData {
    pub fn new(transform: Mat4, color: Vec4) -> Self {
        Self { transform, color }
    }
}

impl Renderer {
    /// Create an instance batch for efficient rendering
    pub fn create_instance_batch(&self, mesh_name: &str, material_name: &str) -> InstanceBatch {
        InstanceBatch {
            mesh_name: mesh_name.to_string(),
            material_name: material_name.to_string(),
            instances: Vec::new(),
        }
    }

    /// Add instance to batch
    pub fn add_instance(&self, batch: &mut InstanceBatch, transform: Mat4, color: Vec4) {
        batch.instances.push(InstanceData::new(transform, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new();

        // Check that default resources were created
        assert!(renderer.material_manager.get_material("InstancedMaterial").is_some());
        assert!(renderer.mesh_manager.get_mesh("Cube").is_some());
        assert!(renderer.shader_manager.get_program("InstancedShader").is_some());
        assert!(renderer.texture_manager.get_texture("placeholder").is_some());
    }

    #[test]
    fn test_render_commands() {
        let mut renderer = Renderer::new();

        // Add some render commands
        renderer.draw_mesh(
            "Cube",
            "InstancedMaterial",
            Mat4::IDENTITY,
            Vec4::new(1.0, 0.0, 0.0, 1.0)
        );

        renderer.draw_mesh(
            "Cube",
            "InstancedMaterialTransparent",
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            Vec4::new(0.0, 1.0, 0.0, 0.5)
        );

        assert_eq!(renderer.render_queue.len(), 2);

        // Test that rendering fails without camera
        assert!(renderer.render_frame().is_err());

        // Add camera and test rendering
        let camera = Camera::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        renderer.set_camera(camera);

        assert!(renderer.render_frame().is_ok());
        assert_eq!(renderer.render_queue.len(), 0); // Should be cleared
    }

    #[test]
    fn test_render_pass() {
        let mut pass = RenderPass::new("MainPass")
            .with_clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));

        let command = RenderCommand {
            mesh_name: "Cube".to_string(),
            material_name: "InstancedMaterial".to_string(),
            transform: Mat4::IDENTITY,
            color: Vec4::ONE,
        };

        pass.add_command(command);

        assert_eq!(pass.name, "MainPass");
        assert_eq!(pass.commands.len(), 1);
    }

    #[test]
    fn test_instance_batch() {
        let renderer = Renderer::new();
        let mut batch = renderer.create_instance_batch("Cube", "InstancedMaterial");

        renderer.add_instance(&mut batch, Mat4::IDENTITY, Vec4::ONE);
        renderer.add_instance(&mut batch, Mat4::from_translation(Vec3::X), Vec4::new(1.0, 0.0, 0.0, 1.0));

        assert_eq!(batch.instances.len(), 2);
        assert_eq!(batch.mesh_name, "Cube");
        assert_eq!(batch.material_name, "InstancedMaterial");
    }

    #[test]
    fn test_render_stats() {
        let renderer = Renderer::new();
        let stats = renderer.get_stats();

        // Should have default resources
        assert!(stats.materials_count >= 2); // InstancedMaterial + Transparent
        assert!(stats.meshes_count >= 1);    // Cube
        assert!(stats.shaders_count >= 1);   // InstancedShader
        assert!(stats.textures_count >= 2);  // placeholder + checkerboard
        assert_eq!(stats.queued_commands, 0);
    }
}
