use crate::font::pdf_font::Font;
use crate::geom::matrix::Matrix;

#[derive(Debug, Clone, Default)]
pub enum TextRenderingMode {
    #[default]
    Fill,
    Stroke,
    FillStroke,
    INVisible,
    FillClip,
    StrokeClip,
    FillStrokeClip,
    Clip,
}

#[derive(Debug, Clone, Default)]
pub struct TextState {
    pub font_size: f64,
    pub char_space: f64,
    pub word_space: f64,
    pub render_mode: TextRenderingMode,
    pub font: Option<Font>,
    pub text_rise: f64,
    pub text_horz_scale: f64,
    pub text_leading: f64,
    text_matrix: Matrix,
    text_line_matrix: Matrix,
}

impl TextState {
    pub fn set_font_size(&mut self, font_size: f64) {
        self.font_size = font_size
    }
    pub fn font_size(&self) -> f64 {
        self.font_size
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = Some(font);
    }

    pub fn font(&self) -> &Font {
        self.font.as_ref().unwrap()
    }

    pub fn set_char_space(&mut self, char_space: f64) {
        self.char_space = char_space
    }
    pub fn char_space(&self) -> f64 {
        self.char_space
    }

    pub fn set_word_space(&mut self, word_space: f64) {
        self.word_space = word_space
    }

    pub fn word_space(&self) -> f64 {
        self.word_space
    }

    pub fn set_render_mode(&mut self, mode: i64) {
        match mode {
            0 => self.render_mode = TextRenderingMode::Fill,
            1 => self.render_mode = TextRenderingMode::Stroke,
            2 => self.render_mode = TextRenderingMode::FillStroke,
            3 => self.render_mode = TextRenderingMode::INVisible,
            4 => self.render_mode = TextRenderingMode::FillClip,
            5 => self.render_mode = TextRenderingMode::StrokeClip,
            6 => self.render_mode = TextRenderingMode::FillStrokeClip,
            7 => self.render_mode = TextRenderingMode::Clip,
            _ => {}
        }
    }
    pub fn render_mode(&mut self) -> &TextRenderingMode {
        &self.render_mode
    }

    pub fn set_text_rise(&mut self, rise: f64) {
        self.text_rise = rise;
    }

    pub fn text_rise(&self) -> f64 {
        self.text_rise
    }
    pub fn set_text_horz_scale(&mut self, horz_scale: f64) {
        self.text_horz_scale = horz_scale;
    }
    pub fn text_horz_scale(&self) -> f64 {
        self.text_horz_scale
    }
    pub fn set_text_leading(&mut self, leading: f64) {
        self.text_leading = leading
    }

    pub fn text_leading(&self) -> f64 {
        self.text_leading
    }

    pub fn text_matrix(&self) -> &Matrix {
        &self.text_matrix
    }

    pub fn text_line_matrix(&self) -> &Matrix {
        &self.text_line_matrix
    }

    pub fn set_text_matrix(&mut self, matrix: Matrix) {
        self.text_matrix = matrix;
    }

    pub fn set_text_line_matrix(&mut self, matrix: Matrix) {
        self.text_line_matrix = matrix;
    }
}
