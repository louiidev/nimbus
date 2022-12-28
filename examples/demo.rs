use glam::{Vec2, Vec3};
use guacamole::{
    components::sprite::{Sprite, SpriteBundle},
    ecs::prelude::Res,
    rect::Rect,
    transform::Transform,
    winit::event::VirtualKeyCode,
    App,
};

fn main() {
    let image = include_bytes!("happy-tree.png");
    let tilemap = include_bytes!("tilemap.png");

    let mut app = App::new(guacamole::window::WindowDescriptor {
        title: "app".to_string(),
        width: 1280.,
        height: 720.,
        ..Default::default()
    });

    let texture_id = app.load_texture(image);
    let timemap_id = app.load_texture(tilemap);

    let sprite = Sprite::new(texture_id);

    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            texture_id: timemap_id,
            texture_rect: Some(Rect {
                min: Vec2::new(0., 160.),
                max: Vec2::new(16., 176.),
            }),
            custom_size: Some(Vec2::new(320., 320.)),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),

        ..Default::default()
    };

    app.spawn_bundle(sprite_bundle).run();
}
