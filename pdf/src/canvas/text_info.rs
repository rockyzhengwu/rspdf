use freetype::Bitmap;

use crate::canvas::graphics_state::GraphicsState;
use crate::geom::rectangle::Rectangle;
use crate::object::PDFString;

// TODO charactor glyph cache

pub struct TextInfo {
    characters: PDFString,
    state: GraphicsState,
    bbox: Rectangle,
}

impl TextInfo {
    pub fn new(characters: PDFString, state: GraphicsState, bbox: Rectangle) -> Self {
        TextInfo {
            characters,
            state,
            bbox,
        }
    }
    pub fn get_unicode(&self) -> String {
        self.state.font().get_unicode(&self.characters)
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

    pub fn position(&self) -> (f64, f64) {
        let tx = self.state.text_matrix().v31;
        let ty = self.state.text_matrix().v32;
        (tx, ty)
    }

    pub fn get_content_width(&self) -> f64 {
        let mut total = 0.0;
        for code in self.characters.bytes() {
            let w = self.state.font().get_width(&(code.to_owned() as u32));
            total += (w as f64 / 1000.0) * self.state.font_size() + self.state.char_spacing()
        }
        total
    }

    pub fn cids(&mut self) -> Vec<u32> {
        self.state.font().code_to_gids(self.characters.bytes())
    }

    pub fn get_glyph(&mut self, code: u32, scale: f64) -> Option<Bitmap> {
        let font_size = self.state.font_size();
        let sx = self.state.text_matrix().v11 * font_size * scale;
        let sy = self.state.text_matrix().v22 * font_size * scale;
        self.state
            .font()
            .decode_to_glyph(code, sx as u32, sy as u32)
    }

    pub fn get_character_width(&self, code: u32) -> f64 {
        ((self.state.font().get_width(&code) as f64 / 1000.0) * self.state.font_size()
            + self.state.char_spacing())
            * self.state.text_matrix().v11
    }

    pub fn bbox(&self) -> &Rectangle {
        &self.bbox
    }
}
