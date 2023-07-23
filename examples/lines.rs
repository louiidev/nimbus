use glam::Vec2;
use nimbus::{
    components::{color::Color, line::Line2D},
    rect::Rect,
    sprite::Sprite,
    utils::collisions::{line_line_collision, line_rectangle_collision},
    Engine, Nimbus,
};

fn main() {
    let app = Engine::default();
    let game = GameExample::default();
    app.run(game);
}

#[derive(Debug, Default)]
pub struct GameExample {
    rect: Rect,
    mouse_line: Line2D,
}

impl Nimbus for GameExample {
    fn init(&mut self, engine: &mut Engine) {
        let handle = engine.load_texture("cloud.png");

        self.rect = Rect::from_center_size(Vec2::new(0., 150.), Vec2::splat(50.));

        self.mouse_line = Line2D(Vec2::ZERO, Vec2::new(50., 50.));
    }

    fn update(&mut self, engine: &mut Engine, delta: f32) {
        if let Some(pos) = engine
            .camera
            .viewport_to_world_position(engine.input.mouse_position, engine.get_viewport())
        {
            self.mouse_line.1 = pos.truncate();
        }
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        if let Some(collision) = line_rectangle_collision(self.mouse_line, self.rect) {
            renderer.draw_line(&self.mouse_line, &Color::RED);
            renderer.draw_line_rect(
                &Rect::from_center_size(collision, Vec2::splat(5.)),
                &Color::RED,
            )
        } else {
            renderer.draw_line(&self.mouse_line, &Color::GREEN);
        }
        renderer.draw_line_rect(&self.rect, &Color::BLUE);
    }
}
