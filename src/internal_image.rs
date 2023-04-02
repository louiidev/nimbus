#[derive(Debug)]
pub struct InternalImage {
    pub data: Vec<u8>,
    pub size: (usize, usize),
}

impl InternalImage {
    pub fn new(data: Vec<u8>, size: (usize, usize)) -> Self {
        Self { data, size }
    }
}
