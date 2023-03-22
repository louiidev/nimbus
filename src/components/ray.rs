use glam::{Vec2, Vec3};

use super::rect::Rect;

pub struct Ray {
    pub origin: Vec2,
    pub direction: Vec2,
}

pub struct Ray3D {
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct RaycastHitInfo {
    pub contact_point: Vec2,
    pub contact_normal: Vec2,
}

impl Ray {}

pub struct Raycast2D {
    pub origin: Vec2,    // Starting point of the ray.
    pub direction: Vec2, // Direction vector of the ray.
}

impl Raycast2D {
    pub fn get_rect_intersect(&self, rect: &Rect) -> Option<RaycastHitInfo> {
        let mut t_near = (rect.min - self.origin) / self.direction;
        let mut t_far = (rect.max - self.origin) / self.direction;

        if t_near.x > t_far.x {
            let (near_x, far_x) = (t_far.x, t_near.x);
            t_near.x = near_x;
            t_far.x = far_x;
        }

        if t_near.y > t_far.y {
            let (near_y, far_y) = (t_far.y, t_near.y);
            t_near.y = near_y;
            t_far.y = far_y;
        }

        if t_near.x > t_far.y || t_near.y > t_far.x {
            return None;
        }

        let t_hit_near = t_near.x.max(t_near.y);
        let t_hit_far = t_near.x.max(t_far.y);

        if t_hit_far < 0f32 || t_hit_near > 1f32 {
            return None;
        }

        let contact_normal = if t_near.x > t_near.y {
            if self.direction.x < 0f32 {
                Vec2::X
            } else {
                -Vec2::X
            }
        } else if self.direction.y < 0f32 {
            Vec2::Y
        } else {
            -Vec2::Y
        };

        Some(RaycastHitInfo {
            contact_point: self.origin + t_hit_near * self.direction,
            contact_normal,
        })
    }
}
