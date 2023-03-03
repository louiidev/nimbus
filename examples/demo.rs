use bevy_ecs::system::ResMut;
use glam::{Vec2, Vec3};
use guacamole::{
    color::Color,
    components::sprite::{Sprite, SpriteBundle},
    rect::Rect,
    texture_atlas::TextureAtlas,
    transform::Transform,
    ui::{button::Button, UiHandler},
    App,
};

fn test_drawing_ui(mut ui_handler: ResMut<UiHandler>) {
    // ui_handler
    ui_handler.layout(Vec2::new(250., 500.), 15.0, |ui| {
        if ui
            .button(Button {
                text: "It's a button",
                ..Default::default()
            })
            .clicked
        {
            dbg!("PRESSED");
        }

        // ui.image();

        ui.button(Button {
            text: "Press me",
            ..Default::default()
        });
    });
}

fn main() {
    let tilemap = include_bytes!("tilemap.png");

    let mut app = App::new(guacamole::window::WindowDescriptor {
        title: "🥑 guacamole 🥑".to_string(),
        width: 1280.,
        height: 720.,
        ..Default::default()
    });

    let timemap_id = app.load_texture(tilemap);

    let texture_atlas = TextureAtlas::from_texture_atlas(
        timemap_id,
        Vec2::new(203., 186.),
        12,
        11,
        Some(Vec2::splat(1.)),
        None,
    );

    let position = Vec3::new(250., 250., 0.);
    let mut count = 0.;

    // for texture in texture_atlas.textures {
    //     dbg!(&texture);
    //     let sprite_bundle = SpriteBundle {
    //         sprite: Sprite {
    //             texture_id: timemap_id,
    //             texture_rect: Some(texture),
    //             custom_size: Some(Vec2::new(120., 120.)),
    //             color: Color::WHITE,
    //             ..Default::default()
    //         },
    //         transform: Transform::from_translation(
    //             Vec3::new(50. * count, 50. * count, 0.) + position,
    //         ),

    //         ..Default::default()
    //     };
    //     count += 1.0;
    //     app = app.spawn_bundle(sprite_bundle)
    // }

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
        // .spawn_bundle(sprite_bundle)
        // .add_system(test_drawing_ui)
        .run();
}
