use std::{collections::BTreeSet, env};

use glam::{Quat, Vec2, Vec3};
use nimbus::{
    camera::Camera,
    components::color::Color,
    cube::Cube,
    input::Input,
    material::Material,
    mesh::{Mesh, MeshAttribute},
    model::Model,
    sprite::Sprite,
    transform::Transform,
    Engine, Nimbus,
};
use wgpu::{include_wgsl, ShaderModuleDescriptor};

fn main() {
    env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"));
    let app = Engine::default();
    let game = GameExample::default();
    app.run(game);
}

#[derive(Debug, Default)]
pub struct GameExample {
    player: (Cube, Transform),
    model: Model,
    camera_pitch: f32,
    camera_yaw: f32,
    last_mouse_motion: Vec2,
}

impl Nimbus for GameExample {
    fn init(&mut self, engine: &mut Engine) {
        let handle = engine.load_texture("container.jpg");

        engine.camera.transform =
            Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);

        self.model = engine.load_obj("models/tile.obj");

        self.player.0.texture = handle;
    }

    fn update(&mut self, engine: &mut Engine, delta: f32) {}

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        // renderer.draw_cube(&self.player.0, self.player.1)

        for x in 0..10 {
            for y in 0..10 {
                renderer.draw_model(
                    &self.model,
                    Transform::from_position(Vec3::new(x as f32 * 2.0, 0., y as f32 * 2.0)),
                );
            }
        }

        // renderer.draw_mesh(&self.mesh);
        // renderer.draw_mesh(&self.mesh);
        // renderer.draw_mesh(&self.mesh);
    }
}
