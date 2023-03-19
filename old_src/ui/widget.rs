use crate::renderer::RenderBatchMeta;

use super::{UiHandler, UiVertex};

pub struct WidgetResponse {
    pub clicked: bool,
}

pub trait Widget {
    fn ui(&mut self, ui: &mut UiHandler) -> WidgetResponse;
    fn get_render_meta(&self) -> RenderBatchMeta<UiVertex>;
}
