use crate::geom::{matrix::Matrix, path::Path};

#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
pub struct GraphicsState {
    ctm: Matrix,
    color_space: Vec<String>,
    color: String,
    char_spacing: f64,
    word_spacing: f64,
    hcaling: f64,
    text_leading: f64,
    font_size: f64,
    text_rise: f64,
    text_knockout: bool,
    font: String,
    line_width: f64,
    line_cap_style: i64,
    line_join: i64,
    line_miter_limit: i64,
    line_dash_pattern: Vec<f64>,
    rendering_intent: i64,
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

    pub fn set_line_dash_pattern(&mut self, pattern: Vec<f64>) {
        self.line_dash_pattern = pattern;
    }

    pub fn set_line_cap_style(&mut self, style: i64) {
        self.line_cap_style = style;
    }

    pub fn set_line_join(&mut self, join: i64) {
        self.line_join = join;
    }

    pub fn set_line_miter_limit(&mut self, limit: i64) {
        self.line_miter_limit = limit;
    }

    pub fn update_ctm_matrix(&mut self, mat: &Matrix) {
        self.ctm = mat.mutiply(&self.ctm);
    }

    pub fn set_char_spacing(&mut self, spacing: f64) {
        self.char_spacing = spacing;
    }

    pub fn set_word_spacing(&mut self, spacing: f64) {
        self.word_spacing = spacing;
    }

    pub fn set_hscaling(&mut self, hscaling: f64) {
        self.hcaling = hscaling;
    }

    pub fn set_text_leading(&mut self, leading: f64) {
        self.text_leading = leading;
    }
    pub fn set_font(&mut self, font: String, size: f64) {
        self.font_size = size;
        self.font = font;
    }

    pub fn set_rendering_indent(&mut self, indent: i64) {
        self.rendering_intent = indent;
    }

    pub fn set_text_rise(&mut self, rise: f64) {
        self.text_rise = rise;
    }

    pub fn set_line_width(&mut self, width: f64) {
        self.line_width = width;
    }

    pub fn text_leading(&self) -> f64 {
        self.text_leading
    }

    pub fn font_size(&self) -> f64 {
        self.font_size
    }

    pub fn font(&self) -> &str {
        &self.font
    }

    pub fn char_spacing(&self) -> f64 {
        self.char_spacing
    }

    pub fn word_spacing(&self) -> f64 {
        self.word_spacing
    }

    pub fn hscaling(&self) -> f64 {
        self.hcaling
    }
}

#[cfg(test)]
mod tests {
    use super::GraphicsState;

    #[test]
    fn test_graphics_state() {
        let gst = GraphicsState::default();
        println!("{:?}", gst);
    }
}
