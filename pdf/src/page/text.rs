use crate::font::pdf_font::Font;
use crate::geom::matrix::Matrix;
use crate::geom::point::Point;
use crate::page::graphics_state::GraphicsState;

#[derive(Debug)]
pub struct TextOpItem {
    bytes: Vec<u8>,
    pos: Option<f64>,
}
impl TextOpItem {
    pub fn new(bytes: Vec<u8>, pos: Option<f64>) -> Self {
        TextOpItem { bytes, pos }
    }
}

pub struct TextItem {
    charcode: u32,
    position: Point,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Text {
    content: Vec<TextOpItem>,
    graphics_state: GraphicsState,
    char_codecs: Vec<u32>,
}

impl Text {
    pub fn new(content: Vec<TextOpItem>, graphics_state: GraphicsState) -> Self {
        let mut char_codecs = Vec::new();
        for con in content.iter() {
            //let char_infos = graphics_state
            //    .text_state
            //    .font()
            //    .decode_charcodes(&con.bytes);
            //for char in char_infos {
            //    char_codecs.push(char.cid().to_owned());
            //}
        }
        Text {
            content,
            graphics_state,
            char_codecs,
        }
    }

    pub fn font(&self) -> &Font {
        self.graphics_state.text_state.font()
    }

    pub fn font_size(&self) -> f64 {
        self.graphics_state.text_state.font_size()
    }

    pub fn ctm(&self) -> &Matrix {
        self.graphics_state.ctm()
    }

    pub fn get_text_matrix(&self) -> Matrix {
        let mut matrix = self.graphics_state.text_state.text_matrix().to_owned();
        for charcode in self.char_codecs.iter() {
            // let width = self.graphics_state.text_state.font().get_width(charcode);
            //let mat = Matrix::new_translation_matrix(width, 0.0);
            //matrix = mat.mutiply(&matrix);
            // TODO vertical
        }
        matrix
    }

    pub fn char_codecs(&self) -> &[u32] {
        self.char_codecs.as_slice()
    }
}
