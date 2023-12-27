use freetype::Bitmap;

use crate::canvas::graphics_state::GraphicsState;
use crate::canvas::matrix::Matrix;
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
    pub fn get_unicode(&self) -> String {
        self.state
            .font()
            .get_unicode(self.characters.binary_bytes().as_slice())
    }

    pub fn content_bytes(&self) -> &[u8] {
        self.characters.bytes()
    }

    pub fn decoded_character(&self) -> Vec<u32> {
        let mut chs = Vec::new();
        for c in self.characters.bytes() {
            chs.push(c.to_owned() as u32);
        }
        chs
    }

    pub fn get_content_width(&self) -> f64 {
        let mut total = 0.0;
        let cids = self.cids();
        for code in cids {
            let w = self.state.font().get_width(&code);
            total += (w * 0.001) * self.state.font_size();
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

    pub fn get_glyph(&mut self, code: u32, scale: f64) -> Option<Bitmap> {
        let font_size = self.state.font_size();
        let sx = self.text_matrix.v11.abs() * font_size * scale;
        let sy = self.text_matrix.v22.abs() * font_size * scale;
        self.state
            .font()
            .decode_to_glyph(code, sx as u32, sy as u32)
    }

    pub fn get_character_width(&self, code: u32) -> f64 {
        (self.state.font().get_width(&code) / 1000.0) * self.state.font_size()
            + self.state.char_spacing()
    }

    pub fn adjust_tmx(&mut self, tx: f64) {
        let mat = Matrix::new_translation_matrix(tx, 0.0);
        self.text_matrix = mat.mutiply(&self.text_matrix);
    }

    pub fn position(&self) -> (f64, f64) {
        let tx = self.text_matrix.v31;
        let ty = self.text_matrix.v32;
        (tx, ty)
    }

    pub fn out_pos(&mut self, cid: u32, ctm: &Matrix) -> (u32, u32) {
        let x = self.text_matrix.v31;
        let y = self.text_matrix.v32;
        let ox: u32 = (x * ctm.v11 + y * ctm.v21 + ctm.v31) as u32;
        let oy: u32 = (x * ctm.v12 + y * ctm.v22 + ctm.v32) as u32;
        let w = self.get_character_width(cid);
        self.adjust_tmx(w);
        (ox, oy)
    }
}
