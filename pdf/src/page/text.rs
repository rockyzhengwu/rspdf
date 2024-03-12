use crate::font::pdf_font::Font;
use crate::geom::matrix::Matrix;
use crate::geom::point::Point;
use crate::page::graphics_state::GraphicsState;

#[derive(Debug)]
pub struct TextOpItem {
    bytes: Vec<u8>,
    adjust: Option<f64>,
}
impl TextOpItem {
    pub fn new(bytes: Vec<u8>, adjust: Option<f64>) -> Self {
        TextOpItem { bytes, adjust }
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
        Text {
            content,
            graphics_state,
            char_codecs: Vec::new(),
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
        let mut text_matrix = self.graphics_state.text_state.text_matrix().to_owned();
        // 需要在这里执行一次
        let font_size = self.graphics_state.text_state.font_size();
        let font = self.graphics_state.text_state.font();
        let char_spacing = self.graphics_state.text_state.char_space();
        let horz_scale = self.graphics_state.text_state.text_horz_scale();
        let mut word_spacing = 0;

        for con in self.content.iter() {}
        text_matrix
    }

    pub fn char_codecs(&self) -> &[u32] {
        self.char_codecs.as_slice()
    }
}
