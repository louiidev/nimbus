use bevy_ecs::{
    prelude::Component,
    system::{Query, Res, ResMut},
};
use glam::{Quat, Vec2, Vec3};
use guacamole::{
    camera::{Camera, CameraBundle},
    color::Color,
    components::{
        sprite::{Sprite, SpriteBundle},
        text::{Text, TextAlignment, TextTheme},
    },
    editor::{Editor, EditorMode},
    resources::inputs::InputController,
    texture_atlas::TextureAtlas,
    transform::{GlobalTransform, Transform},
    ui::{button::Button, UiHandler},
    App, ClearColor,
};

fn test_drawing_ui(
    mut ui_handler: ResMut<UiHandler>,
    mut editor: ResMut<Editor>,
    input: Res<InputController>,
) {
    let button_text = if editor.mode == EditorMode::Game {
        "Switch to Editor"
    } else {
        "Switch to Game"
    };
    // ui_handler
    ui_handler.layout(Vec2::new(0.0, 0.0), 15.0, |ui| {
        // if ui
        //     .button(Button {
        //         text: button_text,
        //         ..Default::default()
        //     })
        //     .clicked
        // {
        //     editor.toggle();
        // }

        ui.text(
            Text {
                value: format!(
                    "Mouse pos x: {}, pos y: {}",
                    input.mouse_position.x, input.mouse_position.y
                ),
                alignment: TextAlignment::default(),
                theme: TextTheme {
                    font_size: 24.,
                    color: Color::WHITE,
                },
                ..Default::default()
            },
            Vec2::new(0., 0.),
            None,
        );
    });
}

fn grid_highlighter(
    mut ui_handler: ResMut<UiHandler>,
    hexagon_query: Query<(&HexagonTile, &Transform)>,
) {
    for (hexagon, transform) in hexagon_query.iter() {}
}

#[derive(Component)]
pub struct HexagonTile {
    pub x: u32,
    pub y: u32,
}

fn main() {
    let mut app = App::new(guacamole::window::WindowDescriptor {
        title: "🥑 guacamole 🥑".to_string(),
        width: 1280.,
        height: 720.,
        ..Default::default()
    });

    let tilemap = include_bytes!("fullTiles.png");

    let timemap_id = app.load_texture(tilemap);

    let texture_atlas = TextureAtlas::from_texture_atlas(
        timemap_id,
        Vec2::new(65., 89.),
        4,
        5,
        Some(Vec2::splat(0.)),
        None,
    );

    let tile_count = 10;

    // for y in 0..tile_count {
    //     for x in 0..tile_count {
    //         let offset_x = if y % 2 == 0 { 65. / 2. } else { 0f32 };

    //         app.world.spawn((
    //             SpriteBundle {
    //                 sprite: Sprite {
    //                     texture_id: timemap_id,
    //                     texture_rect: Some(*texture_atlas.textures.get(0).unwrap()),
    //                     // custom_size: Some(Vec2::new(120., 120.)),
    //                     color: Color::WHITE,
    //                     ..Default::default()
    //                 },
    //                 transform: Transform {
    //                     translation: Vec3::new(
    //                         65. * x as f32 + offset_x,
    //                         -((89. / 2.) * y as f32),
    //                         0.,
    //                     ),
    //                     scale: Vec3::splat(1.),
    //                     ..Default::default()
    //                 },
    //                 ..Default::default()
    //             },
    //             HexagonTile { x, y },
    //         ));
    //     }
    // }

    let tilemap = include_bytes!("tilemap.png");

    let timemap_id = app.load_texture(tilemap);

    let texture_atlas = TextureAtlas::from_texture_atlas(
        timemap_id,
        Vec2::new(16., 16.),
        12,
        11,
        Some(Vec2::splat(1.)),
        None,
    );

    app.world.spawn(SpriteBundle {
        sprite: Sprite {
            texture_id: timemap_id,
            texture_rect: Some(*texture_atlas.textures.get(85).unwrap()),
            // custom_size: Some(Vec2::new(120., 120.)),
            color: Color::WHITE,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(292.5, -235., 1.),
            scale: Vec3::splat(3.),
            ..Default::default()
        },
        ..Default::default()
    });

    app.add_global_system(test_drawing_ui)
        .spawn(CameraBundle {
            transform: Transform {
                translation: Vec3::new(300., -250., 999.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb_u8(152, 110, 69)))
        .run();
}
