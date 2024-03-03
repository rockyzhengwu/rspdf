use crate::geom::matrix::Matrix;
use crate::geom::point::Point;
use crate::object::{PDFDictionary, PDFObject};
use crate::page::graphics_state::GraphicsState;

#[derive(Debug, Default, Clone)]
pub struct AllState {
    graphics_sate: GraphicsState,
    text_matrix: Matrix,
    ctm: Matrix,
    text_pos: Point,
}

impl AllState {
    pub fn update_ext_state(&mut self, ext_obj: PDFObject) {
        // TODO implement update current state
        if let PDFObject::Dictionary(d) = ext_obj {}
    }
    pub fn set_text_matrix(&mut self, matrix: Matrix) {
        self.text_matrix = matrix;
    }
}
