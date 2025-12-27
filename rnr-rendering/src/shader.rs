use std::collections::HashMap;
use glam::{Vec3, Vec4};

/// Shader program containing vertex and fragment shaders
#[derive(Debug, Clone)]
pub struct ShaderProgram {
    pub name: String,
    pub vertex_source: String,
    pub fragment_source: String,
    pub uniforms: HashMap<String, UniformInfo>,
    pub attributes: HashMap<String, AttributeInfo>,
}

#[derive(Debug, Clone)]
pub struct UniformInfo {
    pub name: String,
    pub uniform_type: UniformType,
    pub location: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AttributeInfo {
    pub name: String,
    pub attribute_type: AttributeType,
    pub location: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniformType {
    Matrix4,
    Vec4,
    Vec3,
    Vec2,
    Float,
    Int,
    Sampler2D,
    SamplerCube,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeType {
    Vec4,
    Vec3,
    Vec2,
    Float,
}

/// Shader manager for handling shader resources
pub struct ShaderManager {
    pub programs: HashMap<String, ShaderProgram>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
        }
    }

    /// Register a shader program
    pub fn register_program(&mut self, program: ShaderProgram) {
        self.programs.insert(program.name.clone(), program);
    }

    /// Get a shader program by name
    pub fn get_program(&self, name: &str) -> Option<&ShaderProgram> {
        self.programs.get(name)
    }

    /// Create the instanced shader program (equivalent to InstancedShader from original RNR)
    pub fn create_instanced_shader(&mut self) {
        let vertex_source = r#"
// Instanced Vertex Shader - Rust implementation of original InstancedShader.vert
#version 450

layout(location = 0) in vec4 vertex;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 uv0;
layout(location = 3) in vec4 uv1;
layout(location = 4) in vec4 uv2;
layout(location = 5) in vec4 uv3;
layout(location = 6) in vec4 uv4;
layout(location = 7) in vec3 tangent;

layout(location = 0) out vec2 v_uv0;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec3 v_world_pos;
layout(location = 3) out vec4 v_part_color;

layout(set = 0, binding = 0) uniform ViewProj {
    mat4 view_proj;
};

void main() {
    // Reconstruct world matrix from UV coordinates (instancing data)
    mat4 world_matrix = mat4(
        uv1,  // column 0
        uv2,  // column 1
        uv3,  // column 2
        vec4(0.0, 0.0, 0.0, 1.0)  // column 3
    );

    vec4 world_pos = vertex * world_matrix;
    vec3 world_norm = normalize(mat3(world_matrix) * normal);

    gl_Position = view_proj * world_pos;

    v_uv0 = uv0.xy;
    v_normal = world_norm;
    v_world_pos = world_pos.xyz;
    v_part_color = uv4;
}
"#.to_string();

        let fragment_source = r#"
// Instanced Fragment Shader - Rust implementation of original InstancedShader.frag
#version 450

layout(location = 0) in vec2 v_uv0;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec3 v_world_pos;
layout(location = 3) in vec4 v_part_color;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform Lighting {
    vec4 light_position;
    vec3 camera_position;
    vec3 light_ambient;
    vec3 light_diffuse;
    vec3 light_specular;
    float light_gloss;
};

void main() {
    vec4 base_color = v_part_color;

    // Blinn-Phong lighting
    vec3 normal = normalize(v_normal);
    vec3 light_dir = light_position.xyz - v_world_pos * light_position.w;
    vec3 eye_dir = normalize(camera_position - v_world_pos);

    float light_distance = length(light_dir);
    light_dir = normalize(light_dir);

    float n_dot_l = max(0.0, dot(normal, light_dir));
    vec3 half_vector = normalize(light_dir + eye_dir);
    float h_dot_n = max(0.0, dot(half_vector, normal));

    vec3 ambient = light_ambient;
    vec3 diffuse = light_diffuse * n_dot_l;
    vec3 specular = light_specular * pow(h_dot_n, light_gloss);

    vec3 direct_lighting = diffuse + specular;

    out_color = vec4(base_color.xyz * (direct_lighting + ambient), base_color.a);
}
"#.to_string();

        let mut uniforms = HashMap::new();
        uniforms.insert("view_proj".to_string(), UniformInfo {
            name: "view_proj".to_string(),
            uniform_type: UniformType::Matrix4,
            location: Some(0),
        });
        uniforms.insert("lighting".to_string(), UniformInfo {
            name: "lighting".to_string(),
            uniform_type: UniformType::Vec4, // This is a uniform block
            location: Some(1),
        });

        let mut attributes = HashMap::new();
        attributes.insert("vertex".to_string(), AttributeInfo {
            name: "vertex".to_string(),
            attribute_type: AttributeType::Vec4,
            location: Some(0),
        });
        attributes.insert("normal".to_string(), AttributeInfo {
            name: "normal".to_string(),
            attribute_type: AttributeType::Vec3,
            location: Some(1),
        });
        attributes.insert("uv0".to_string(), AttributeInfo {
            name: "uv0".to_string(),
            attribute_type: AttributeType::Vec4,
            location: Some(2),
        });
        attributes.insert("uv1".to_string(), AttributeInfo {
            name: "uv1".to_string(),
            attribute_type: AttributeType::Vec4,
            location: Some(3),
        });
        attributes.insert("uv2".to_string(), AttributeInfo {
            name: "uv2".to_string(),
            attribute_type: AttributeType::Vec4,
            location: Some(4),
        });
        attributes.insert("uv3".to_string(), AttributeInfo {
            name: "uv3".to_string(),
            attribute_type: AttributeType::Vec4,
            location: Some(5),
        });
        attributes.insert("uv4".to_string(), AttributeInfo {
            name: "uv4".to_string(),
            attribute_type: AttributeType::Vec4,
            location: Some(6),
        });
        attributes.insert("tangent".to_string(), AttributeInfo {
            name: "tangent".to_string(),
            attribute_type: AttributeType::Vec3,
            location: Some(7),
        });

        let program = ShaderProgram {
            name: "InstancedShader".to_string(),
            vertex_source,
            fragment_source,
            uniforms,
            attributes,
        };

        self.register_program(program);
    }
}

impl Default for ShaderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderProgram {
    /// Create a new shader program
    pub fn new(name: &str, vertex_source: &str, fragment_source: &str) -> Self {
        Self {
            name: name.to_string(),
            vertex_source: vertex_source.to_string(),
            fragment_source: fragment_source.to_string(),
            uniforms: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    /// Add a uniform
    pub fn with_uniform(mut self, name: &str, uniform_type: UniformType, location: Option<u32>) -> Self {
        self.uniforms.insert(name.to_string(), UniformInfo {
            name: name.to_string(),
            uniform_type,
            location,
        });
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, name: &str, attribute_type: AttributeType, location: Option<u32>) -> Self {
        self.attributes.insert(name.to_string(), AttributeInfo {
            name: name.to_string(),
            attribute_type,
            location,
        });
        self
    }

    /// Get uniform info
    pub fn get_uniform(&self, name: &str) -> Option<&UniformInfo> {
        self.uniforms.get(name)
    }

    /// Get attribute info
    pub fn get_attribute(&self, name: &str) -> Option<&AttributeInfo> {
        self.attributes.get(name)
    }
}

/// Lighting uniform buffer data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LightingUniforms {
    pub light_position: Vec4,
    pub camera_position: Vec3,
    pub _padding1: f32,
    pub light_ambient: Vec3,
    pub _padding2: f32,
    pub light_diffuse: Vec3,
    pub _padding3: f32,
    pub light_specular: Vec3,
    pub light_gloss: f32,
}

impl Default for LightingUniforms {
    fn default() -> Self {
        Self {
            light_position: Vec4::new(10.0, 10.0, 10.0, 1.0),
            camera_position: Vec3::ZERO,
            _padding1: 0.0,
            light_ambient: Vec3::new(0.2, 0.2, 0.2),
            _padding2: 0.0,
            light_diffuse: Vec3::new(0.8, 0.8, 0.8),
            _padding3: 0.0,
            light_specular: Vec3::new(1.0, 1.0, 1.0),
            light_gloss: 12.5,
        }
    }
}

unsafe impl bytemuck::Pod for LightingUniforms {}
unsafe impl bytemuck::Zeroable for LightingUniforms {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_program_creation() {
        let program = ShaderProgram::new(
            "TestShader",
            "vertex shader source",
            "fragment shader source"
        )
        .with_uniform("view_proj", UniformType::Matrix4, Some(0))
        .with_attribute("position", AttributeType::Vec3, Some(0));

        assert_eq!(program.name, "TestShader");
        assert!(program.get_uniform("view_proj").is_some());
        assert!(program.get_attribute("position").is_some());
    }

    #[test]
    fn test_shader_manager() {
        let mut manager = ShaderManager::new();

        let program = ShaderProgram::new("Test", "vert", "frag");
        manager.register_program(program);

        assert!(manager.get_program("Test").is_some());
        assert!(manager.get_program("NonExistent").is_none());
    }

    #[test]
    fn test_instanced_shader_creation() {
        let mut manager = ShaderManager::new();
        manager.create_instanced_shader();

        let program = manager.get_program("InstancedShader").unwrap();
        assert!(program.vertex_source.contains("#version 450"));
        assert!(program.fragment_source.contains("#version 450"));
        assert!(program.get_attribute("vertex").is_some());
        assert!(program.get_uniform("view_proj").is_some());
    }
}
