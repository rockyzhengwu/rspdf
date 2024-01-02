use crate::canvas::graphics_state::GraphicsState;

#[allow(dead_code)]
pub struct ImageInfo {
    state: GraphicsState,
    width: f64,
    height: f64,
    color: String,
    data: Option<Vec<u8>>,
    bpc: u64,
}

impl ImageInfo {
}
