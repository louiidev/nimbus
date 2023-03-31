use nimbus::{window::WindowDescriptor, Engine, Nimbus};

pub struct UiExample {}

impl Nimbus for UiExample {
    fn init(&mut self, engine: &mut nimbus::Engine) {}

    fn update(&mut self, engine: &mut nimbus::Engine, _delta: f32) {
        engine.ui.left_panel(200., |ui| {
            ui.label("My Name is ted");
        });
        engine.ui.panel(|ui| {
            ui.label("Helloooooo");
        });
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, delta: f32) {}
}

fn main() {
    let engine = Engine::new(WindowDescriptor::default());

    engine.run(UiExample {});
}
