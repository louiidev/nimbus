use crate::renderer::Renderer;

pub fn post_render(renderer: &mut Renderer) {
    renderer.render_batch_ui.clear();
    renderer.render_batch_debug.clear();
}
