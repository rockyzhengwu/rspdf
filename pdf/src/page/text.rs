use crate::color::ColorSpace;
use crate::font::pdf_font::Font;
use crate::geom::matrix::Matrix;
use crate::page::graphics_state::GraphicsState;
use crate::page::graphics_state::TextRenderingMode;

#[derive(Debug)]
pub struct TextOpItem {
    bytes: Vec<u8>,
    adjust: Option<f64>,
}
impl TextOpItem {
    pub fn new(bytes: Vec<u8>, adjust: Option<f64>) -> Self {
        TextOpItem { bytes, adjust }
    }
    pub fn adjust(&self) -> f64 {
        self.adjust.map_or(0.0, |x| x.to_owned())
    }
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Text {
    content: Vec<TextOpItem>,
    graphics_state: GraphicsState,
}

impl Text {
    pub fn new(content: Vec<TextOpItem>, graphics_state: GraphicsState) -> Self {
        Text {
            content,
            graphics_state,
        }
    }
    pub fn fill_color(&self) -> &ColorSpace {
        &self.graphics_state.fill_color
    }

    pub fn stroke_color(&self) -> &ColorSpace {
        &self.graphics_state.stroke_color
    }

    pub fn text_items(&self) -> &[TextOpItem] {
        self.content.as_slice()
    }

    pub fn font(&self) -> &Font {
        self.graphics_state.font()
    }

    pub fn font_size(&self) -> f64 {
        self.graphics_state.font_size
    }

    pub fn ctm(&self) -> &Matrix {
        &self.graphics_state.ctm
    }

    pub fn text_matrix(&self) -> &Matrix {
        &self.graphics_state.text_matrix
    }

    pub fn char_spacing(&self) -> f64 {
        self.graphics_state.char_space
    }

    pub fn text_horz_scale(&self) -> f64 {
        self.graphics_state.text_horz_scale
    }

    pub fn word_space(&self) -> f64 {
        self.graphics_state.word_space
    }
    pub fn text_rise(&self) -> f64 {
        self.graphics_state.text_rise
    }

    pub fn get_text_matrix(&self) -> Matrix {
        let mut text_matrix = self.graphics_state.text_matrix.to_owned();
        let font_size = self.graphics_state.font_size;
        let font = self.graphics_state.font();
        let char_spacing = self.graphics_state.char_space;
        let word_spacing = self.graphics_state.word_space;

        for con in self.content.iter() {
            let chars = font.decode_chars(con.bytes());
            let mut displacement = 0.0;
            let tj = con.adjust();
            if font.is_vertical() {
                let rm = Matrix::new_translation_matrix(0.0, -tj * 0.001);
                text_matrix = rm.mutiply(&text_matrix);
            } else {
                let rm = Matrix::new_translation_matrix(-tj * 0.001, 0.0);
                text_matrix = rm.mutiply(&text_matrix);
            }
            for char in chars.iter() {
                displacement += font.get_char_width(char) * 0.001 * font_size + char_spacing;
                if char.is_space() {
                    displacement += word_spacing;
                }
            }
            // TODO handler vertical
            // println!("{:?}",font.wmode);
            if font.is_vertical() {
                let trm = Matrix::new_translation_matrix(0.0, displacement);
                text_matrix = trm.mutiply(&text_matrix);
            } else {
                let trm = Matrix::new_translation_matrix(displacement, 0.0);
                text_matrix = trm.mutiply(&text_matrix);
            }
        }
        text_matrix
    }

    pub fn render_mode(&self) -> &TextRenderingMode {
        &self.graphics_state.render_mode
    }
}
