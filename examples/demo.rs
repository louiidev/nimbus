use bevy_ecs::system::ResMut;
use glam::{Vec2, Vec3};
use guacamole::{
    color::Color,
    components::sprite::{Sprite, SpriteBundle},
    rect::Rect,
    transform::Transform,
    ui::{button::Button, UiHandler},
    App,
};

fn test_drawing_ui(mut ui_handler: ResMut<UiHandler>) {
    // ui_handler
    ui_handler.layout(Vec2::ZERO, 5.0, |ui| {
        if ui
            .button(Button {
                text: "hello",
                ..Default::default()
            })
            .clicked
        {
            dbg!("PRESSED");
        }

        ui.button(Button {
            text: "hello",
            ..Default::default()
        });
    });
}

fn main() {
    let tilemap = include_bytes!("tilemap.png");

    let mut app = App::new(guacamole::window::WindowDescriptor {
        title: "app".to_string(),
        width: 1280.,
        height: 720.,
        ..Default::default()
    });

    let timemap_id = app.load_texture(tilemap);

    // let texture_atlas =
    //     TextureAtlas::from_texture_atlas(timemap_id, Vec2::, 12, 11, 1, 0.);

    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            texture_id: timemap_id,
            texture_rect: Some(Rect {
                min: Vec2::new(0., 156.),
                max: Vec2::new(16., 170.),
            }),
            custom_size: Some(Vec2::new(320., 320.)),
            color: Color::WHITE,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(250., 250., 0.)),

        ..Default::default()
    };

    app.add_system(test_drawing_ui)
        .spawn_bundle(sprite_bundle)
        // .add_system(test_drawing_ui)
        .run();
}
