use glam::Vec3;
use rapier3d::prelude::RigidBodyHandle;

/// Joint service for managing physics joints (simplified for now)
pub struct JointsService {
    /// Number of ball joints (spherical joints)
    pub ball_joints_count: usize,
    /// Number of fixed joints
    pub fixed_joints_count: usize,
    /// Number of prismatic joints (sliding joints)
    pub prismatic_joints_count: usize,
    /// Number of revolute joints (hinge joints)
    pub revolute_joints_count: usize,
}

impl JointsService {
    /// Create a new joints service
    pub fn new() -> Self {
        Self {
            ball_joints_count: 0,
            fixed_joints_count: 0,
            prismatic_joints_count: 0,
            revolute_joints_count: 0,
        }
    }

    /// Add a ball joint
    pub fn add_ball_joint(&mut self) {
        self.ball_joints_count += 1;
    }

    /// Add a fixed joint
    pub fn add_fixed_joint(&mut self) {
        self.fixed_joints_count += 1;
    }

    /// Add a prismatic joint
    pub fn add_prismatic_joint(&mut self) {
        self.prismatic_joints_count += 1;
    }

    /// Add a revolute joint
    pub fn add_revolute_joint(&mut self) {
        self.revolute_joints_count += 1;
    }

    /// Clear all joints
    pub fn clear(&mut self) {
        self.ball_joints_count = 0;
        self.fixed_joints_count = 0;
        self.prismatic_joints_count = 0;
        self.revolute_joints_count = 0;
    }

    /// Get total number of joints
    pub fn joint_count(&self) -> usize {
        self.ball_joints_count +
        self.fixed_joints_count +
        self.prismatic_joints_count +
        self.revolute_joints_count
    }
}

impl Default for JointsService {
    fn default() -> Self {
        Self::new()
    }
}

/// Weld constraint for rigid body connections
pub struct Weld {
    /// Position of the weld
    pub position: Vec3,
    /// Parent rigid body handle
    pub parent_handle: Option<RigidBodyHandle>,
    /// Child rigid body handle
    pub child_handle: Option<RigidBodyHandle>,
    /// Weld strength
    pub strength: f32,
}

impl Weld {
    /// Create a new weld
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            parent_handle: None,
            child_handle: None,
            strength: 1.0,
        }
    }

    /// Set the parent rigid body
    pub fn set_parent(&mut self, handle: RigidBodyHandle) {
        self.parent_handle = Some(handle);
    }

    /// Set the child rigid body
    pub fn set_child(&mut self, handle: RigidBodyHandle) {
        self.child_handle = Some(handle);
    }

    /// Set weld strength
    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joints_service() {
        let mut service = JointsService::new();

        assert_eq!(service.joint_count(), 0);

        // Add a ball joint
        service.add_ball_joint();

        assert_eq!(service.joint_count(), 1);
        assert_eq!(service.ball_joints_count, 1);

        service.clear();
        assert_eq!(service.joint_count(), 0);
    }

    #[test]
    fn test_weld() {
        let mut weld = Weld::new(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(weld.position, Vec3::new(1.0, 2.0, 3.0));
        assert!(weld.parent_handle.is_none());
        assert!(weld.child_handle.is_none());
        assert_eq!(weld.strength, 1.0);

        // Mock rigid body handles (would be created by physics world)
        let parent_handle = RigidBodyHandle::from_raw_parts(0, 0);
        let child_handle = RigidBodyHandle::from_raw_parts(1, 0);

        weld.set_parent(parent_handle);
        weld.set_child(child_handle);
        weld.set_strength(0.8);

        assert!(weld.parent_handle.is_some());
        assert!(weld.child_handle.is_some());
        assert_eq!(weld.strength, 0.8);
    }
}
