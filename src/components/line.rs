use glam::{Vec2, Vec3};

pub struct Line(Vec3, Vec3);

#[derive(Default, Debug, Clone, Copy)]
pub struct Line2D(pub Vec2, pub Vec2);
