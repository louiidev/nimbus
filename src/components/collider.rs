use bevy_ecs::{
    prelude::{Component, Entity},
    query::{Changed, Without},
    system::{Local, Query, Res},
};
use glam::Vec2;

use crate::{
    camera::Camera,
    color::Color,
    rect::Rect,
    resources::inputs::InputController,
    transform::{self, GlobalTransform, Transform},
    utils::collision,
};

use super::{parent::Parent, sprite::Sprite};

pub type Size = Vec2;
pub type Radius = f32;

#[derive(Clone, Copy, Debug)]
pub enum Collider2DShape {
    Rect(Size),
    Circle(Radius),
}

impl Default for Collider2DShape {
    fn default() -> Self {
        Collider2DShape::Rect(Vec2::default())
    }
}

#[derive(Default, Component, Clone, Copy, Debug)]
pub struct Collider2D {
    pub shape: Collider2DShape,
}

#[derive(Default, Component, Clone, Copy, Debug)]
pub struct DebugCollider(Collider2D);

// pub fn debug_collider_propagate_system(
//     mut root_query: Query<
//         (
//             Changed<GlobalTransform>,
//             &GlobalTransform,
//             &mut DebugCollider,
//             Entity,
//         ),
//         Without<Parent>,
//     >,
// ) {
//     for (transform_changed, global_transform, mut debug_collider, entity) in root_query.iter_mut() {
//         let mut changed = transform_changed;
//         if transform_changed {
//             *debug_collider = DebugCollider {}
//         }
//     }
// }

#[derive(Default)]
pub struct PickedEntity {
    entity: Option<(Entity, Rect)>,
}

pub fn debug_collider_picker(
    mut query: Query<(&mut Sprite, &Transform, Entity)>,
    mut picked: Local<PickedEntity>,
    input_controller: Res<InputController>,
) {
    let mouse_just_pressed = input_controller
        .mouse_button_inputs
        .just_pressed(winit::event::MouseButton::Left);

    let mouse_down = input_controller
        .mouse_button_inputs
        .pressed(winit::event::MouseButton::Left);

    if picked.entity.is_none() && !mouse_just_pressed || picked.entity.is_some() && mouse_down {
        return;
    }

    for (mut sprite, transform, entity) in query.iter_mut() {
        let sprite_rect = sprite.texture_rect.unwrap_or(Rect::default());
        let sprite_pos = transform.translation.truncate();

        if mouse_just_pressed {
            dbg!(sprite_rect.size() * transform.scale.truncate(), sprite_pos);
        }

        if mouse_just_pressed
            && collision::rect_contains_point(
                sprite_rect.size() * transform.scale.truncate(),
                sprite_pos,
                input_controller.mouse_position,
            )
        {
            sprite.color = Color::BLUE;
            picked.entity = Some((
                entity,
                Rect {
                    min: sprite_pos + sprite_rect.min,
                    max: sprite_pos + sprite_rect.max,
                },
            ));
            return;
        }
    }
}
