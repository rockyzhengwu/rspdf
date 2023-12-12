use crate::canvas::graphics_state::GraphicsState;
use crate::geom::{path::Path, rectangle::Rectangle};

#[allow(dead_code)]
#[derive(Debug)]
pub struct PathInfo {
    path: Path,
    state: GraphicsState,
    bbox: Rectangle,
}

impl PathInfo {
    pub fn new(path: Path, state: GraphicsState, bbox: Rectangle) -> Self {
        PathInfo { path, state, bbox }
    }
}
