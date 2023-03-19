use bevy_ecs::{
    prelude::{Component, Entity},
    query::{Changed, Without},
    system::{Local, Query, Res, ResMut},
};
use glam::Vec2;

use crate::{
    camera::Camera,
    color::Color,
    components::sprite::Anchor,
    rect::Rect,
    renderer::{debug_drawing::DebugMesh, texture::Texture},
    resources::{
        inputs::InputController,
        utils::{Assets, ResourceVec},
    },
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
    textures: Res<Assets<Texture>>,
    mut debug_meshes: ResMut<ResourceVec<DebugMesh>>,
) {
    let mouse_just_pressed = input_controller
        .mouse_button_inputs
        .just_pressed(winit::event::MouseButton::Left);

    let mouse_down = input_controller
        .mouse_button_inputs
        .pressed(winit::event::MouseButton::Left);

    for (mut sprite, transform, entity) in query.iter_mut() {
        let texture = textures.data.get(&sprite.texture_id).unwrap();
        let current_image_size =
            Vec2::new(texture.dimensions.0 as f32, texture.dimensions.1 as f32);

        // By default, the size of the quad is the size of the texture
        let mut quad_size = current_image_size;

        // If a rect is specified, adjust UVs and the size of the quad
        if let Some(rect) = sprite.texture_rect {
            quad_size = rect.size();
        }

        // Override the size if a custom one is specified
        if let Some(custom_size) = sprite.custom_size {
            quad_size = custom_size;
        }

        let sprite_pos = transform.translation.truncate();

        let quad_size = quad_size * transform.scale.truncate();

        let rect = Rect {
            min: sprite_pos - quad_size / 2.,
            max: sprite_pos + quad_size / 2.,
        };

        // if let Some(picked) = picked.entity {
        //     debug_meshes.values.push(DebugMesh::from(picked.1));
        // }

        // if collision::rect_contains_point(rect.size(), rect.min, input_controller.mouse_position) {
        //     sprite.color = Color::new(sprite.color.red, sprite.color.green, sprite.color.blue, 0.5);
        //     picked.entity = Some((entity, rect));
        //     return;
        // }
    }
}
