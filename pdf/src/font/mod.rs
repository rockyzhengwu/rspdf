use std::fmt::Display;

mod afm;
pub mod builtin_font;
pub mod cid_font;
pub mod cmap;
pub mod descriptor;
pub mod encoding;
pub mod font_program;
pub mod glyph_name;
pub mod open_type;
pub mod open_type_program;
pub mod pdf_font;
pub mod simple_font;
pub mod to_unicode;
pub mod truetype;
pub mod type0;
pub mod type1_program;
pub mod type1c;

#[derive(Debug)]
pub enum WritingMode {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct CharCode {
    code: u32,
    length: u8,
    width: f32,
    origin_x: f32,
    origin_y: f32,
}

impl CharCode {
    pub fn new(code: u32, length: u8, width: f32) -> Self {
        Self {
            code,
            length,
            width,
            origin_x: 0.0,
            origin_y: 0.0,
        }
    }
    pub fn set_with(&mut self, width: f32) {
        self.width = width;
    }

    pub fn length(&self) -> u8 {
        self.length
    }
    pub fn code(&self) -> u32 {
        self.code
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn set_origin_x(&mut self, x: f32) {
        self.origin_x = x;
    }
    pub fn set_origin_y(&mut self, y: f32) {
        self.origin_y = y;
    }
}

#[derive(Debug)]
pub enum GlyphDesc {
    Name(String),
    Gid(u32),
}

impl Display for GlyphDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlyphDesc::Name(s) => write!(f, "{}", s),
            GlyphDesc::Gid(gid) => write!(f, "{}", gid),
        }
    }
}
