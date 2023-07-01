use glam::Vec2;
use nimbus::{
    components::color::Color, utils::collisions::rect_rect_collision, Engine, Nimbus, Rect,
};

fn main() {
    let app = Engine::default();
    let game = GameExample::default();
    app.run(game);
}

#[derive(Debug, Default)]
pub struct GameExample {
    rect: Rect,
    mouse_rect: Rect,
}

impl Nimbus for GameExample {
    fn init(&mut self, _engine: &mut Engine) {
        self.rect = Rect::from_center_size(Vec2::new(0., 150.), Vec2::splat(50.));

        self.mouse_rect = Rect::from_center_size(Vec2::ZERO, Vec2::splat(50.));
    }

    fn update(&mut self, engine: &mut Engine, _delta: f32) {
        if let Some(pos) = engine
            .camera
            .viewport_to_world_position(engine.input.mouse_position, engine.get_viewport())
        {
            let new_rect = Rect::from_center_size(pos.truncate(), Vec2::splat(50.));

            if !rect_rect_collision(new_rect, self.rect) {
                self.mouse_rect = new_rect;
            }
        }
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        let color = if rect_rect_collision(self.mouse_rect, self.rect) {
            Color::RED
        } else {
            Color::GREEN
        };
        renderer.draw_line_rect(self.mouse_rect, color);
        renderer.draw_line_rect(self.rect, Color::BLUE);
    }
}
