use glam::{Mat4, Vec3, Quat};

/// Camera for 3D rendering
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,        // Field of view in degrees
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub projection_matrix: Mat4,
    pub view_matrix: Mat4,
    pub view_projection_matrix: Mat4,
    pub needs_update: bool,
}

impl Camera {
    /// Create a new camera at position looking at target
    pub fn new(position: Vec3, target: Vec3) -> Self {
        let mut camera = Self {
            position,
            rotation: Quat::IDENTITY,
            fov: 60.0,
            aspect_ratio: 16.0 / 9.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            projection_matrix: Mat4::IDENTITY,
            view_matrix: Mat4::IDENTITY,
            view_projection_matrix: Mat4::IDENTITY,
            needs_update: true,
        };

        camera.look_at(target);
        camera.update_matrices();

        camera
    }

    /// Create a perspective camera
    pub fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let mut camera = Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            fov,
            aspect_ratio,
            near_plane: near,
            far_plane: far,
            projection_matrix: Mat4::IDENTITY,
            view_matrix: Mat4::IDENTITY,
            view_projection_matrix: Mat4::IDENTITY,
            needs_update: true,
        };

        camera.update_matrices();
        camera
    }

    /// Look at a target point
    pub fn look_at(&mut self, target: Vec3) {
        let forward = (target - self.position).normalize();
        let right = Vec3::Y.cross(forward).normalize();
        let up = forward.cross(right).normalize();

        // Create rotation matrix and convert to quaternion
        let rotation_matrix = Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            forward.extend(0.0),
            Vec3::ZERO.extend(1.0),
        );

        self.rotation = Quat::from_mat4(&rotation_matrix);
        self.needs_update = true;
    }

    /// Set camera position
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.needs_update = true;
    }

    /// Set camera rotation
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.needs_update = true;
    }

    /// Move camera by offset
    pub fn translate(&mut self, offset: Vec3) {
        self.position += offset;
        self.needs_update = true;
    }

    /// Rotate camera by quaternion
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
        self.needs_update = true;
    }

    /// Set field of view
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.needs_update = true;
    }

    /// Set aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.needs_update = true;
    }

    /// Set clipping planes
    pub fn set_clipping_planes(&mut self, near: f32, far: f32) {
        self.near_plane = near;
        self.far_plane = far;
        self.needs_update = true;
    }

    /// Get the forward direction vector
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    /// Get the right direction vector
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction vector
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Update view and projection matrices if needed
    pub fn update_matrices(&mut self) {
        if !self.needs_update {
            return;
        }

        // Update projection matrix
        self.projection_matrix = Mat4::perspective_rh(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.near_plane,
            self.far_plane,
        );

        // Update view matrix
        let translation = Mat4::from_translation(-self.position);
        let rotation = Mat4::from_quat(self.rotation.conjugate());
        self.view_matrix = rotation * translation;

        // Update combined view-projection matrix
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;

        self.needs_update = false;
    }

    /// Get the current view matrix
    pub fn view_matrix(&self) -> Mat4 {
        self.view_matrix
    }

    /// Get the current projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection_matrix
    }

    /// Get the current view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.view_projection_matrix
    }

    /// Convert world point to screen space (NDC)
    pub fn world_to_screen(&self, world_point: Vec3) -> Vec3 {
        let clip_space = self.view_projection_matrix * world_point.extend(1.0);
        let ndc = clip_space / clip_space.w;

        // Convert to screen space (0 to 1)
        Vec3::new(
            (ndc.x + 1.0) * 0.5,
            (1.0 - ndc.y) * 0.5, // Flip Y axis
            ndc.z,
        )
    }

    /// Convert screen point to world ray
    pub fn screen_to_world_ray(&self, screen_x: f32, screen_y: f32) -> (Vec3, Vec3) {
        // Convert screen coordinates to NDC (-1 to 1)
        let ndc_x = screen_x * 2.0 - 1.0;
        let ndc_y = (1.0 - screen_y) * 2.0 - 1.0; // Flip Y axis

        // Create ray in clip space
        let clip_ray = Vec3::new(ndc_x, ndc_y, -1.0); // -1 for forward direction

        // Transform to world space
        let inv_view_proj = self.view_projection_matrix.inverse();
        let world_near = inv_view_proj * clip_ray.extend(1.0);
        let world_far = inv_view_proj * clip_ray.extend(0.0); // Point at far plane

        let origin = world_near.truncate() / world_near.w;
        let direction = (world_far.truncate() / world_far.w - origin).normalize();

        (origin, direction)
    }

    /// Check if a point is visible by the camera
    pub fn is_point_visible(&self, point: Vec3) -> bool {
        let screen_pos = self.world_to_screen(point);

        // Check if point is within NDC bounds (-1 to 1 for x,y and 0 to 1 for z)
        screen_pos.x >= -1.0 && screen_pos.x <= 1.0 &&
        screen_pos.y >= -1.0 && screen_pos.y <= 1.0 &&
        screen_pos.z >= 0.0 && screen_pos.z <= 1.0
    }

    /// Get camera frustum corners in world space
    pub fn get_frustum_corners(&self) -> [Vec3; 8] {
        let near_height = 2.0 * (self.fov.to_radians() * 0.5).tan() * self.near_plane;
        let near_width = near_height * self.aspect_ratio;
        let far_height = 2.0 * (self.fov.to_radians() * 0.5).tan() * self.far_plane;
        let far_width = far_height * self.aspect_ratio;

        let forward = self.forward();
        let right = self.right();
        let up = self.up();

        let near_center = self.position + forward * self.near_plane;
        let far_center = self.position + forward * self.far_plane;

        [
            // Near plane corners
            near_center - right * near_width * 0.5 - up * near_height * 0.5,
            near_center + right * near_width * 0.5 - up * near_height * 0.5,
            near_center + right * near_width * 0.5 + up * near_height * 0.5,
            near_center - right * near_width * 0.5 + up * near_height * 0.5,
            // Far plane corners
            far_center - right * far_width * 0.5 - up * far_height * 0.5,
            far_center + right * far_width * 0.5 - up * far_height * 0.5,
            far_center + right * far_width * 0.5 + up * far_height * 0.5,
            far_center - right * far_width * 0.5 + up * far_height * 0.5,
        ]
    }
}

/// Orthographic camera for 2D rendering or isometric views
#[derive(Debug, Clone)]
pub struct OrthographicCamera {
    pub position: Vec3,
    pub rotation: Quat,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub projection_matrix: Mat4,
    pub view_matrix: Mat4,
    pub view_projection_matrix: Mat4,
    pub needs_update: bool,
}

impl OrthographicCamera {
    /// Create a new orthographic camera
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let mut camera = Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            left,
            right,
            bottom,
            top,
            near_plane: near,
            far_plane: far,
            projection_matrix: Mat4::IDENTITY,
            view_matrix: Mat4::IDENTITY,
            view_projection_matrix: Mat4::IDENTITY,
            needs_update: true,
        };

        camera.update_matrices();
        camera
    }

    /// Update matrices
    pub fn update_matrices(&mut self) {
        if !self.needs_update {
            return;
        }

        self.projection_matrix = Mat4::orthographic_rh(
            self.left, self.right, self.bottom, self.top, self.near_plane, self.far_plane
        );

        let translation = Mat4::from_translation(-self.position);
        let rotation = Mat4::from_quat(self.rotation.conjugate());
        self.view_matrix = rotation * translation;

        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
        self.needs_update = false;
    }

    /// Get view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.view_projection_matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.fov, 60.0);
        assert_eq!(camera.aspect_ratio, 16.0 / 9.0);
        assert_eq!(camera.near_plane, 0.1);
        assert_eq!(camera.far_plane, 1000.0);
    }

    #[test]
    fn test_perspective_camera() {
        let camera = Camera::perspective(90.0, 1.0, 0.1, 100.0);

        assert_eq!(camera.fov, 90.0);
        assert_eq!(camera.aspect_ratio, 1.0);
        assert_eq!(camera.near_plane, 0.1);
        assert_eq!(camera.far_plane, 100.0);
    }

    #[test]
    fn test_camera_look_at() {
        let mut camera = Camera::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        camera.look_at(Vec3::new(1.0, 0.0, 0.0));
        camera.update_matrices();

        // Forward vector should point towards the target
        let forward = camera.forward();
        assert!(forward.x > 0.0); // Should be pointing in positive X direction
    }

    #[test]
    fn test_camera_transform() {
        let mut camera = Camera::new(Vec3::ZERO, Vec3::Z);

        camera.translate(Vec3::new(1.0, 2.0, 3.0));
        camera.update_matrices();

        assert_eq!(camera.position, Vec3::new(1.0, 2.0, 3.0));

        let rotation = Quat::from_rotation_y(PI * 0.5);
        camera.rotate(rotation);
        camera.update_matrices();

        // Should be rotated 90 degrees around Y axis
        let forward = camera.forward();
        assert!((forward.x - 1.0).abs() < 0.001); // Should be pointing in X direction
        assert!(forward.z.abs() < 0.001); // Z component should be near zero
    }

    #[test]
    fn test_world_to_screen() {
        let camera = Camera::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        // Point at origin should be at center of screen
        let screen_pos = camera.world_to_screen(Vec3::ZERO);
        assert!((screen_pos.x - 0.5).abs() < 0.001);
        assert!((screen_pos.y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_screen_to_world_ray() {
        let camera = Camera::new(Vec3::ZERO, Vec3::Z);

        // Center of screen should create ray from origin forward
        let (origin, direction) = camera.screen_to_world_ray(0.5, 0.5);

        assert!(origin.length() < 0.001); // Should start near origin
        assert!((direction.z + 1.0).abs() < 0.001); // Should point forward (negative Z)
    }

    #[test]
    fn test_visibility_check() {
        let camera = Camera::new(Vec3::ZERO, Vec3::Z);

        // Point in front should be visible
        assert!(camera.is_point_visible(Vec3::new(0.0, 0.0, -1.0)));

        // Point behind should not be visible
        assert!(!camera.is_point_visible(Vec3::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn test_orthographic_camera() {
        let mut camera = OrthographicCamera::new(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);

        camera.update_matrices();

        let view_proj = camera.view_projection_matrix();
        assert!(view_proj != Mat4::IDENTITY);
    }

    #[test]
    fn test_frustum_corners() {
        let camera = Camera::new(Vec3::ZERO, Vec3::Z);

        let corners = camera.get_frustum_corners();

        // Should have 8 corners
        assert_eq!(corners.len(), 8);

        // Near plane corners should be closer than far plane corners
        for i in 0..4 {
            assert!(corners[i].z.abs() < corners[i + 4].z.abs());
        }
    }
}
