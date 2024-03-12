use crate::font::pdf_font::Font;
use crate::geom::matrix::Matrix;
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
    pub fn adjust(&self) -> Option<&f64> {
        self.adjust.as_ref()
    }
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
        let font_size = self.graphics_state.text_state.font_size();
        let font = self.graphics_state.text_state.font();
        let char_spacing = self.graphics_state.text_state.char_space();
        let horz_scale = self.graphics_state.text_state.text_horz_scale();
        let word_spacing = self.graphics_state.text_state.word_space();

        for con in self.content.iter() {
            let unicode = font.to_unicode(&con.bytes);
            if let Some(adjust) = con.adjust() {
                let tj = adjust * -1.0 * font_size * 0.001 * horz_scale;
                let translate = Matrix::new_translation_matrix(tj, 0.0);
                text_matrix = translate.mutiply(&text_matrix);
            }
            let mut total_width = 0.0;
            let chars = font.decode_chars(&con.bytes);
            for char in chars.iter() {
                total_width += font.get_char_width(char) * 0.001 + char_spacing;
                if char.is_space() {
                    total_width += word_spacing;
                }
            }
            let trm = Matrix::new_translation_matrix(total_width, 0.0);
            text_matrix = trm.mutiply(&text_matrix);
        }
        text_matrix
    }

    pub fn char_codecs(&self) -> &[u32] {
        self.char_codecs.as_slice()
    }
}
