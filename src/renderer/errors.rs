use wgpu::{CreateSurfaceError, RequestDeviceError};

#[derive(Debug)]
pub struct RenderError {
    pub message: String,
}

impl RenderError {
    pub fn new(message: impl ToString) -> Self {
        RenderError {
            message: message.to_string(),
        }
    }
}

impl From<CreateSurfaceError> for RenderError {
    fn from(e: CreateSurfaceError) -> RenderError {
        RenderError {
            message: format!("wgpu::CreateSurfaceError {:?}", &e.to_string()),
        }
    }
}

impl From<RequestDeviceError> for RenderError {
    fn from(e: RequestDeviceError) -> RenderError {
        RenderError {
            message: format!("wgpu::RequestDeviceError {:?}", &e.to_string()),
        }
    }
}
