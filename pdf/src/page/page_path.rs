use crate::geom::{matrix::Matrix, path::Path, point::Point};
use crate::page::graphics_state::GraphicsState;

#[derive(Debug)]
pub struct PagePath {
    path: Path,
    graphics_state: GraphicsState,
}

impl PagePath {
    pub fn new(path: Path, graphics_state: GraphicsState) -> Self {
        PagePath {
            path,
            graphics_state,
        }
    }

    pub fn ctm(&self) -> &Matrix {
        self.graphics_state.ctm()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn line_width(&self) -> &f64 {
        self.graphics_state.path_state.line_width()
    }
}
