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
        let mut word_spacing = self.graphics_state.text_state.word_space();

        for con in self.content.iter() {
            let unicode = font.to_unicode(&con.bytes);
            print!("{:?}", unicode);
            let mut total_with = 0.0;
            let chars = font.decode_chars(&con.bytes);
            for char in chars.iter() {
                total_with += (font.get_char_width(char) * 0.001) * font_size;
                if char.is_space() {
                    total_with += word_spacing;
                }
                total_with += char_spacing;
            }
            let trm = Matrix::new_translation_matrix(total_with, 0.0);
            text_matrix = trm.mutiply(&text_matrix);
        }
        println!("{:?}, {:?}", text_matrix.v32, text_matrix.v32);
        text_matrix
    }

    pub fn char_codecs(&self) -> &[u32] {
        self.char_codecs.as_slice()
    }
}
