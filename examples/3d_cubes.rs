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

#[derive(Debug)]
pub struct CubeMat;

impl Material for CubeMat {
    fn use_depth_stencil(&self) -> bool {
        true
    }
    // fn vertex_attributes(&self) -> BTreeSet<MeshAttribute> {
    //     BTreeSet::from([MeshAttribute::Position, MeshAttribute::Color])
    // }
    // fn has_texture(&self) -> bool {
    //     false
    // }

    // fn shader(&self) -> ShaderModuleDescriptor {
    //     include_wgsl!("../src/renderer/default_shaders/test.wgsl")
    // }
}

impl Nimbus for GameExample {
    fn init(&mut self, engine: &mut Engine) {
        let handle = engine.load_texture("container.jpg");

        engine.camera = Camera::perspective(45f32.to_radians(), 0.1, 1280. / 720.);

        engine.renderer.mode_3d = true;

        self.model = engine.load_obj("models/cube.obj");

        let material = CubeMat;

        let mat = engine.renderer.push_material(material);

        self.model.material = mat;

        self.player.0 = Cube::new(Vec3::new(1., 1., 1.), Color::PURPLE, mat);

        self.player.0.texture = handle;
    }

    fn update(&mut self, engine: &mut Engine, delta: f32) {
        let mut move_direction = Vec2::default();
        for key in engine.get_pressed() {
            use nimbus::input::Input::*;
            match key {
                W => move_direction += Vec2::Y,
                S => move_direction -= Vec2::Y,
                A => move_direction -= Vec2::X,
                D => move_direction += Vec2::X,
                _ => {}
            }
        }

        use nimbus::input::Axis::*;
        move_direction += Vec2::new(engine.get_axis(LeftX), engine.get_axis(LeftY));

        if move_direction != Vec2::default() {
            let (yaw_sin, yaw_cos) = self.camera_yaw.sin_cos();
            let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
            let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
            engine.camera.position += forward * move_direction.y * delta * 5.;
            engine.camera.position += right * move_direction.x * delta * 5.;
        }

        if !engine.pressed(Input::MouseButtonLeft) {
            return;
        }

        let mut x_rot = 0f32;
        let mut y_rot = 0f32;

        if self.last_mouse_motion != engine.input.mouse_motion {
            self.last_mouse_motion = engine.input.mouse_motion;
        } else {
            return;
        }

        x_rot += self.last_mouse_motion.x;
        y_rot -= self.last_mouse_motion.y;

        if x_rot == 0. && y_rot == 0. {
            return;
        }

        let mouse_sensitivity = 200.;

        let rotation =
            Quat::from_euler(glam::EulerRot::YXZ, self.camera_yaw, self.camera_pitch, 0.);

        self.camera_yaw += -self.last_mouse_motion.x.to_radians() * mouse_sensitivity * delta;
        self.camera_pitch += -self.last_mouse_motion.y.to_radians() * mouse_sensitivity * delta;

        engine.camera.rotation = rotation;
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        // renderer.draw_cube(&self.player.0, self.player.1)

        renderer.draw_model(&self.model, Transform::IDENTITY);
        renderer.draw_model(&self.model, Transform::from_position(Vec3::Z * -5.));
        renderer.draw_model(
            &self.model,
            Transform::from_position(Vec3::Y * 2. + Vec3::X * 0.5),
        );
        // renderer.draw_mesh(&self.mesh);
        // renderer.draw_mesh(&self.mesh);
        // renderer.draw_mesh(&self.mesh);
    }
}
