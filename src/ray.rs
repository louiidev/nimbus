use glam::Vec3;

/// A ray is an infinite line starting at `origin`, going in `direction`.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    /// The origin of the ray.
    pub origin: Vec3,
    /// A normalized vector representing the direction of the ray.
    pub direction: Vec3,
}
