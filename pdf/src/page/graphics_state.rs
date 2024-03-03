use crate::geom::matrix::Matrix;
use crate::object::PDFDictionary;
use crate::page::general_state::GeneralState;
use crate::page::path_state::PathState;
use crate::page::text_state::TextState;

#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
pub struct GraphicsState {
    pub text_state: TextState,
    pub path_state: PathState,
    pub general_state: GeneralState,
    pub ctm: Matrix,
}

impl GraphicsState {
    pub fn new(ctm: Matrix) -> Self {
        GraphicsState {
            ctm,
            ..Default::default()
        }
    }
    pub fn ctm(&self) -> &Matrix {
        &self.ctm
    }

    pub fn update_ctm_matrix(&mut self, mat: &Matrix) {
        self.ctm = mat.mutiply(&self.ctm);
    }

    pub fn process_ext_gs(&mut self, _obj: PDFDictionary) {
        unimplemented!()
    }
}
