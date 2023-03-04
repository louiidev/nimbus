use bevy_ecs::system::ResMut;
use glam::{Vec2, Vec3};
use guacamole::{
    color::Color,
    components::sprite::{Sprite, SpriteBundle},
    editor::{Editor, EditorMode},
    texture_atlas::TextureAtlas,
    transform::Transform,
    ui::{button::Button, UiHandler},
    App,
};

fn test_drawing_ui(mut ui_handler: ResMut<UiHandler>, mut editor: ResMut<Editor>) {
    let button_text = if editor.mode == EditorMode::Game {
        "Switch to Editor"
    } else {
        "Switch to Game"
    };

    // ui_handler
    ui_handler.layout(Vec2::new(250., 500.), 15.0, |ui| {
        if ui
            .button(Button {
                text: button_text,
                ..Default::default()
            })
            .clicked
        {
            editor.toggle();
        }
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
        Vec2::new(16., 16.),
        12,
        11,
        Some(Vec2::splat(1.)),
        None,
    );

    let position = Vec3::new(0., 0., 0.);

    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            texture_id: timemap_id,
            texture_rect: Some(*texture_atlas.textures.get(85).unwrap()),
            // custom_size: Some(Vec2::new(120., 120.)),
            color: Color::WHITE,
            ..Default::default()
        },
        transform: Transform {
            translation: position,
            scale: Vec3::splat(5.),
            ..Default::default()
        },
        ..Default::default()
    };

    app.add_global_system(test_drawing_ui)
        .spawn_bundle(sprite_bundle)
        .run();
}
