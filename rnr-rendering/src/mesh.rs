use glam::{Vec3, Vec2, Vec4, Mat4};
use std::collections::HashMap;

/// Vertex data for mesh rendering
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv0: Vec2,
    pub uv1: Vec4,  // Used for instancing data
    pub uv2: Vec4,  // Used for instancing data
    pub uv3: Vec4,  // Used for instancing data
    pub uv4: Vec4,  // Used for part color
    pub tangent: Vec3,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position,
            normal,
            uv0: uv,
            uv1: Vec4::ZERO,
            uv2: Vec4::ZERO,
            uv3: Vec4::ZERO,
            uv4: Vec4::new(1.0, 1.0, 1.0, 1.0), // Default white color
            tangent: Vec3::ZERO,
        }
    }

    /// Create vertex with instancing data (equivalent to original RNR instancing)
    pub fn with_instancing(
        position: Vec3,
        normal: Vec3,
        uv: Vec2,
        world_matrix: Mat4,
        color: Vec4,
    ) -> Self {
        // Extract matrix columns for UV storage (original RNR method)
        let uv1 = world_matrix.col(0);
        let uv2 = world_matrix.col(1);
        let uv3 = world_matrix.col(2);
        // uv4 is used for color

        Self {
            position,
            normal,
            uv0: uv,
            uv1,
            uv2,
            uv3,
            uv4: color,
            tangent: Vec3::ZERO,
        }
    }
}

/// Mesh containing vertex and index data
#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounds: BoundingBox,
    pub submeshes: Vec<SubMesh>,
}

#[derive(Debug, Clone)]
pub struct SubMesh {
    pub name: String,
    pub index_start: u32,
    pub index_count: u32,
    pub material_name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn empty() -> Self {
        Self {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn expand(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vertices: Vec::new(),
            indices: Vec::new(),
            bounds: BoundingBox::empty(),
            submeshes: Vec::new(),
        }
    }

    /// Create a cube mesh (equivalent to Cube.mesh from original RNR)
    pub fn create_cube() -> Self {
        let mut mesh = Self::new("Cube");

        // Cube vertices (8 corners)
        let positions = [
            Vec3::new(1.0, 1.0, -1.0),   // 0: front top right
            Vec3::new(1.0, -1.0, -1.0),  // 1: front bottom right
            Vec3::new(1.0, 1.0, 1.0),    // 2: back top right
            Vec3::new(1.0, -1.0, 1.0),   // 3: back bottom right
            Vec3::new(-1.0, 1.0, -1.0),  // 4: front top left
            Vec3::new(-1.0, -1.0, -1.0), // 5: front bottom left
            Vec3::new(-1.0, 1.0, 1.0),   // 6: back top left
            Vec3::new(-1.0, -1.0, 1.0),  // 7: back bottom left
        ];

        // Normals for each face
        let normals = [
            Vec3::new(0.0, 0.0, -1.0), // front
            Vec3::new(0.0, 0.0, 1.0),  // back
            Vec3::new(-1.0, 0.0, 0.0), // left
            Vec3::new(0.0, -1.0, 0.0), // bottom
            Vec3::new(1.0, 0.0, 0.0),  // right
            Vec3::new(0.0, 1.0, 0.0),  // top
        ];

        // UV coordinates for texture mapping
        let uvs = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];

        // Front face (z = -1)
        mesh.add_vertex(Vertex::new(positions[4], normals[0], uvs[0]));
        mesh.add_vertex(Vertex::new(positions[0], normals[0], uvs[1]));
        mesh.add_vertex(Vertex::new(positions[1], normals[0], uvs[2]));
        mesh.add_vertex(Vertex::new(positions[5], normals[0], uvs[3]));

        // Back face (z = 1)
        mesh.add_vertex(Vertex::new(positions[2], normals[1], uvs[0]));
        mesh.add_vertex(Vertex::new(positions[6], normals[1], uvs[1]));
        mesh.add_vertex(Vertex::new(positions[7], normals[1], uvs[2]));
        mesh.add_vertex(Vertex::new(positions[3], normals[1], uvs[3]));

        // Left face (x = -1)
        mesh.add_vertex(Vertex::new(positions[6], normals[2], uvs[0]));
        mesh.add_vertex(Vertex::new(positions[4], normals[2], uvs[1]));
        mesh.add_vertex(Vertex::new(positions[5], normals[2], uvs[2]));
        mesh.add_vertex(Vertex::new(positions[7], normals[2], uvs[3]));

        // Right face (x = 1)
        mesh.add_vertex(Vertex::new(positions[0], normals[4], uvs[0]));
        mesh.add_vertex(Vertex::new(positions[2], normals[4], uvs[1]));
        mesh.add_vertex(Vertex::new(positions[3], normals[4], uvs[2]));
        mesh.add_vertex(Vertex::new(positions[1], normals[4], uvs[3]));

        // Top face (y = 1)
        mesh.add_vertex(Vertex::new(positions[4], normals[5], uvs[0]));
        mesh.add_vertex(Vertex::new(positions[6], normals[5], uvs[1]));
        mesh.add_vertex(Vertex::new(positions[2], normals[5], uvs[2]));
        mesh.add_vertex(Vertex::new(positions[0], normals[5], uvs[3]));

        // Bottom face (y = -1)
        mesh.add_vertex(Vertex::new(positions[5], normals[3], uvs[0]));
        mesh.add_vertex(Vertex::new(positions[1], normals[3], uvs[1]));
        mesh.add_vertex(Vertex::new(positions[3], normals[3], uvs[2]));
        mesh.add_vertex(Vertex::new(positions[7], normals[3], uvs[3]));

        // Create indices for triangles (6 faces * 2 triangles * 3 vertices = 36 indices)
        let indices = [
            // Front face
            0, 1, 2, 0, 2, 3,
            // Back face
            4, 5, 6, 4, 6, 7,
            // Left face
            8, 9, 10, 8, 10, 11,
            // Right face
            12, 13, 14, 12, 14, 15,
            // Top face
            16, 17, 18, 16, 18, 19,
            // Bottom face
            20, 21, 22, 20, 22, 23,
        ];

        mesh.indices.extend_from_slice(&indices);

        // Add submesh
        mesh.add_submesh("Cube", 0, indices.len() as u32, "InstancedMaterial");

        mesh
    }

    /// Add a vertex to the mesh and update bounds
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.bounds.expand(vertex.position);
        self.vertices.push(vertex);
    }

    /// Add a submesh
    pub fn add_submesh(&mut self, name: &str, index_start: u32, index_count: u32, material_name: &str) {
        self.submeshes.push(SubMesh {
            name: name.to_string(),
            index_start,
            index_count,
            material_name: material_name.to_string(),
        });
    }

    /// Calculate vertex normals (if not provided)
    pub fn calculate_normals(&mut self) {
        // Reset normals
        for vertex in &mut self.vertices {
            vertex.normal = Vec3::ZERO;
        }

        // Calculate face normals and accumulate
        for chunk in self.indices.chunks(3) {
            if chunk.len() == 3 {
                let v0 = self.vertices[chunk[0] as usize].position;
                let v1 = self.vertices[chunk[1] as usize].position;
                let v2 = self.vertices[chunk[2] as usize].position;

                let edge1 = v1 - v0;
                let edge2 = v2 - v0;
                let face_normal = edge1.cross(edge2).normalize();

                self.vertices[chunk[0] as usize].normal += face_normal;
                self.vertices[chunk[1] as usize].normal += face_normal;
                self.vertices[chunk[2] as usize].normal += face_normal;
            }
        }

        // Normalize accumulated normals
        for vertex in &mut self.vertices {
            if vertex.normal != Vec3::ZERO {
                vertex.normal = vertex.normal.normalize();
            }
        }
    }

    /// Get the vertex buffer data as bytes
    pub fn vertex_buffer_data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    /// Get the index buffer data as bytes
    pub fn index_buffer_data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }
}

/// Mesh manager for handling mesh resources
pub struct MeshManager {
    pub meshes: HashMap<String, Mesh>,
}

impl MeshManager {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
        }
    }

    /// Register a mesh
    pub fn register_mesh(&mut self, mesh: Mesh) {
        self.meshes.insert(mesh.name.clone(), mesh);
    }

    /// Get a mesh by name
    pub fn get_mesh(&self, name: &str) -> Option<&Mesh> {
        self.meshes.get(name)
    }

    /// Create and register default meshes
    pub fn create_default_meshes(&mut self) {
        let cube_mesh = Mesh::create_cube();
        self.register_mesh(cube_mesh);
    }
}

impl Default for MeshManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = Vertex::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec2::new(0.5, 0.5)
        );

        assert_eq!(vertex.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(vertex.normal, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(vertex.uv0, Vec2::new(0.5, 0.5));
        assert_eq!(vertex.uv4, Vec4::new(1.0, 1.0, 1.0, 1.0)); // Default color
    }

    #[test]
    fn test_cube_mesh_creation() {
        let mesh = Mesh::create_cube();

        assert_eq!(mesh.name, "Cube");
        assert_eq!(mesh.vertices.len(), 24); // 6 faces * 4 vertices
        assert_eq!(mesh.indices.len(), 36);  // 6 faces * 2 triangles * 3 indices
        assert_eq!(mesh.submeshes.len(), 1);

        // Check bounds
        assert!(mesh.bounds.min.x <= -1.0);
        assert!(mesh.bounds.max.x >= 1.0);
        assert!(mesh.bounds.min.y <= -1.0);
        assert!(mesh.bounds.max.y >= 1.0);
        assert!(mesh.bounds.min.z <= -1.0);
        assert!(mesh.bounds.max.z >= 1.0);
    }

    #[test]
    fn test_bounding_box() {
        let mut bbox = BoundingBox::empty();

        bbox.expand(Vec3::new(1.0, 2.0, 3.0));
        bbox.expand(Vec3::new(-1.0, -2.0, -3.0));

        assert_eq!(bbox.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.max, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(bbox.center(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.size(), Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_mesh_manager() {
        let mut manager = MeshManager::new();

        let mesh = Mesh::create_cube();
        manager.register_mesh(mesh);

        assert!(manager.get_mesh("Cube").is_some());
        assert!(manager.get_mesh("NonExistent").is_none());
    }
}
