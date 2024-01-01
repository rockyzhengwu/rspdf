use crate::canvas::graphics_state::GraphicsState;
use crate::canvas::matrix::Matrix;
use crate::font::Font;
use crate::object::PDFString;

pub struct TextInfo {
    characters: PDFString,
    state: GraphicsState,
    text_matrix: Matrix,
}

impl TextInfo {
    pub fn new(characters: PDFString, state: GraphicsState, text_matrix: Matrix) -> Self {
        TextInfo {
            characters,
            state,
            text_matrix,
        }
    }

    pub fn get_unicode(&self, cids: &[u32]) -> String {
        self.state.font().get_unicode(cids)
    }

    pub fn matrix(&self) -> Matrix {
        self.text_matrix.clone()
    }

    pub fn content_bytes(&self) -> &[u8] {
        self.characters.bytes()
    }

    pub fn get_content_width(&self) -> f64 {
        let mut total = 0.0;
        let cids = self.cids();
        for code in cids {
            let w = self.get_character_width(&code);
            total += w;
            if code == 32 {
                total += self.state.word_spacing() * self.state.hscaling() * 0.01;
            }
        }
        total
    }

    pub fn get_ctm(&self) -> &Matrix {
        self.state.ctm()
    }

    pub fn cids(&self) -> Vec<u32> {
        self.state
            .font()
            .code_to_cids(self.characters.binary_bytes().as_slice())
    }

    pub fn font(&self) -> &Font {
        self.state.font()
    }

    pub fn get_character_width(&self, code: &u32) -> f64 {
        (self.state.font().get_width(code) * 0.001) * self.state.font_size()
            + self.state.char_spacing()
    }

    pub fn shift(&mut self, tx: f64) {
        let mat = Matrix::new_translation_matrix(tx, 0.0);
        self.text_matrix = mat.mutiply(&self.text_matrix);
    }

    pub fn text_matrix(&self) -> &Matrix {
        &self.text_matrix
    }

    pub fn position(&self) -> (f64, f64) {
        let tx = self.text_matrix.v31;
        let ty = self.text_matrix.v32;
        (tx, ty)
    }

    pub fn out_pos(&mut self, cid: &u32, ctm: &Matrix) -> (f64, f64) {
        let x = self.text_matrix.v31;
        let y = self.text_matrix.v32;
        let ox = x * ctm.v11 + y * ctm.v21 + ctm.v31;
        let oy = x * ctm.v12 + y * ctm.v22 + ctm.v32;
        let w = self.get_character_width(cid);
        self.shift(w);
        (ox, oy)
    }

    pub fn font_size(&self) -> f64 {
        self.state.font_size()
    }
}
