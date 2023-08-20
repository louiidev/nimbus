use nimbus::{window::WindowDescriptor, Engine, Nimbus};

pub struct UiExample {}

impl Nimbus for UiExample {
    fn init(&mut self, _engine: &mut nimbus::Engine) {}

    fn update(&mut self, engine: &mut nimbus::Engine, _delta: f32) {
        engine.renderer.panel(|ui| {});

        // engine.ui.left_panel(200., |ui| {
        //     ui.label("My Name is ted");
        // });
        // engine.ui.panel(|ui| {
        //     ui.label("Helloooooo");
        // });
    }

    fn render(&mut self, _renderer: &mut nimbus::renderer::Renderer, _delta: f32) {}
}

fn main() {
    let engine = Engine::new(WindowDescriptor::default());

    engine.run(UiExample {});
}
