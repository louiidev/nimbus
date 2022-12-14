use bevy_ecs::prelude::{Component, Query};
use guacamole::{
    renderer::sprite::{Sprite, SpriteBundle},
    transform::Transform,
    App,
};

fn main() {
    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }
    #[derive(Component)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    fn movement(mut query: Query<(&mut Position, &Velocity)>) {
        for (mut position, velocity) in query.iter_mut() {
            position.x += velocity.x;
            position.y += velocity.y;
        }
    }

    let image = include_bytes!("happy-tree.png");

    let mut app = App::new(guacamole::window::WindowDescriptor {
        title: "app".to_string(),
        width: 1280.,
        height: 720.,
        ..Default::default()
    })
    .add_system(movement)
    .init_2d_renderer();

    let texture_id = app.load_texture(image);

    let sprite = Sprite::new(texture_id);

    let sprite_bundle = SpriteBundle {
        sprite,
        ..Default::default()
    };

    app.spawn_bundle(sprite_bundle).run();
}
