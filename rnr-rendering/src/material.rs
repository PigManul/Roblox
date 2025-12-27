use std::collections::HashMap;
use glam::{Vec3, Vec4};

/// Material properties for rendering
#[derive(Debug, Clone)]
pub struct Material {
    /// Material name
    pub name: String,
    /// Base color (RGBA)
    pub base_color: Vec4,
    /// Specular color
    pub specular_color: Vec3,
    /// Specular power (shininess)
    pub specular_power: f32,
    /// Whether the material is transparent
    pub transparent: bool,
    /// Whether to write to depth buffer
    pub depth_write: bool,
    /// Custom properties
    pub properties: HashMap<String, MaterialProperty>,
}

#[derive(Debug, Clone)]
pub enum MaterialProperty {
    Float(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
    Texture(String), // Texture name/path
}

impl Material {
    /// Create a new material with default properties
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            specular_color: Vec3::new(1.0, 1.0, 1.0),
            specular_power: 12.5,
            transparent: false,
            depth_write: true,
            properties: HashMap::new(),
        }
    }

    /// Create a transparent material
    pub fn transparent(name: &str) -> Self {
        Self {
            transparent: true,
            depth_write: false,
            ..Self::new(name)
        }
    }

    /// Set base color
    pub fn with_base_color(mut self, color: Vec4) -> Self {
        self.base_color = color;
        self
    }

    /// Set specular properties
    pub fn with_specular(mut self, color: Vec3, power: f32) -> Self {
        self.specular_color = color;
        self.specular_power = power;
        self
    }

    /// Add a custom property
    pub fn with_property(mut self, name: &str, property: MaterialProperty) -> Self {
        self.properties.insert(name.to_string(), property);
        self
    }

    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&MaterialProperty> {
        self.properties.get(name)
    }

    /// Check if material needs alpha blending
    pub fn needs_alpha_blend(&self) -> bool {
        self.transparent || self.base_color.w < 1.0
    }
}

/// Material technique (equivalent to Ogre technique)
#[derive(Debug, Clone)]
pub struct Technique {
    pub name: String,
    pub passes: Vec<Pass>,
}

impl Technique {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passes: Vec::new(),
        }
    }

    pub fn with_pass(mut self, pass: Pass) -> Self {
        self.passes.push(pass);
        self
    }
}

/// Render pass within a technique
#[derive(Debug, Clone)]
pub struct Pass {
    pub vertex_shader: Option<String>,
    pub fragment_shader: Option<String>,
    pub depth_write: bool,
    pub blend_mode: BlendMode,
    pub parameters: HashMap<String, ShaderParameter>,
}

#[derive(Debug, Clone)]
pub enum BlendMode {
    Opaque,
    AlphaBlend,
    Additive,
}

#[derive(Debug, Clone)]
pub enum ShaderParameter {
    Matrix4(glam::Mat4),
    Vec4(glam::Vec4),
    Vec3(glam::Vec3),
    Vec2(glam::Vec2),
    Float(f32),
    Int(i32),
    Texture(String),
}

impl Pass {
    pub fn new() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            depth_write: true,
            blend_mode: BlendMode::Opaque,
            parameters: HashMap::new(),
        }
    }

    pub fn with_shaders(mut self, vertex: &str, fragment: &str) -> Self {
        self.vertex_shader = Some(vertex.to_string());
        self.fragment_shader = Some(fragment.to_string());
        self
    }

    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode.clone();
        if matches!(blend_mode, BlendMode::AlphaBlend) {
            self.depth_write = false;
        }
        self
    }

    pub fn with_parameter(mut self, name: &str, param: ShaderParameter) -> Self {
        self.parameters.insert(name.to_string(), param);
        self
    }
}

/// Material manager for handling material resources
pub struct MaterialManager {
    pub materials: HashMap<String, Material>,
    pub techniques: HashMap<String, Technique>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            techniques: HashMap::new(),
        }
    }

    /// Register a material
    pub fn register_material(&mut self, material: Material) {
        self.materials.insert(material.name.clone(), material);
    }

    /// Get a material by name
    pub fn get_material(&self, name: &str) -> Option<&Material> {
        self.materials.get(name)
    }

    /// Register a technique
    pub fn register_technique(&mut self, technique: Technique) {
        self.techniques.insert(technique.name.clone(), technique);
    }

    /// Get a technique by name
    pub fn get_technique(&self, name: &str) -> Option<&Technique> {
        self.techniques.get(name)
    }

    /// Create instanced material (equivalent to InstancedMaterial from original RNR)
    pub fn create_instanced_material(&mut self) {
        let material = Material::new("InstancedMaterial")
            .with_specular(Vec3::new(1.0, 1.0, 1.0), 12.5);

        let technique = Technique::new("InstancedMaterial")
            .with_pass(Pass::new()
                .with_shaders("InstancedShader.vert", "InstancedShader.frag")
                .with_parameter("viewProjMatrix", ShaderParameter::Matrix4(glam::Mat4::IDENTITY))
                .with_parameter("lightPosition", ShaderParameter::Vec4(glam::Vec4::ZERO))
                .with_parameter("cameraPosition", ShaderParameter::Vec3(glam::Vec3::ZERO))
                .with_parameter("lightAmbient", ShaderParameter::Vec3(glam::Vec3::new(0.2, 0.2, 0.2)))
                .with_parameter("lightDiffuse", ShaderParameter::Vec3(glam::Vec3::new(0.8, 0.8, 0.8)))
                .with_parameter("lightSpecular", ShaderParameter::Vec3(glam::Vec3::new(1.0, 1.0, 1.0)))
                .with_parameter("lightGloss", ShaderParameter::Float(12.5))
            );

        self.register_material(material);
        self.register_technique(technique);
    }

    /// Create transparent instanced material
    pub fn create_instanced_material_transparent(&mut self) {
        let material = Material::transparent("InstancedMaterialTransparent")
            .with_specular(Vec3::new(1.0, 1.0, 1.0), 12.5);

        let technique = Technique::new("InstancedMaterialTransparent")
            .with_pass(Pass::new()
                .with_shaders("InstancedShader.vert", "InstancedShader.frag")
                .with_blend_mode(BlendMode::AlphaBlend)
                .with_parameter("viewProjMatrix", ShaderParameter::Matrix4(glam::Mat4::IDENTITY))
                .with_parameter("lightPosition", ShaderParameter::Vec4(glam::Vec4::ZERO))
                .with_parameter("cameraPosition", ShaderParameter::Vec3(glam::Vec3::ZERO))
                .with_parameter("lightAmbient", ShaderParameter::Vec3(glam::Vec3::new(0.2, 0.2, 0.2)))
                .with_parameter("lightDiffuse", ShaderParameter::Vec3(glam::Vec3::new(0.8, 0.8, 0.8)))
                .with_parameter("lightSpecular", ShaderParameter::Vec3(glam::Vec3::new(1.0, 1.0, 1.0)))
                .with_parameter("lightGloss", ShaderParameter::Float(12.5))
            );

        self.register_material(material);
        self.register_technique(technique);
    }
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let material = Material::new("TestMaterial")
            .with_base_color(Vec4::new(1.0, 0.0, 0.0, 1.0))
            .with_specular(Vec3::new(1.0, 1.0, 1.0), 32.0);

        assert_eq!(material.name, "TestMaterial");
        assert_eq!(material.base_color, Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(material.specular_power, 32.0);
        assert!(!material.transparent);
    }

    #[test]
    fn test_transparent_material() {
        let material = Material::transparent("TransparentMaterial");

        assert!(material.transparent);
        assert!(!material.depth_write);
        assert!(material.needs_alpha_blend());
    }

    #[test]
    fn test_material_manager() {
        let mut manager = MaterialManager::new();

        let material = Material::new("ManagedMaterial");
        manager.register_material(material);

        assert!(manager.get_material("ManagedMaterial").is_some());
        assert!(manager.get_material("NonExistent").is_none());
    }
}
