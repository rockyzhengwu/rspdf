use std::io::{Read, Seek};

use freetype::GlyphSlot;

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::cmap::charcode::CharCode;
use crate::font::composite_font::{load_composite_font, CompositeFont};
use crate::font::simple_font::{load_simple_font, SimpleFont};
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub enum Font {
    Simple(SimpleFont),
    Composite(CompositeFont),
}

impl Font {
    pub fn to_unicode(&self, bytes: &[u8]) -> Vec<String> {
        match self {
            Font::Simple(sf) => sf.decode_to_unicode(bytes),
            Font::Composite(cf) => cf.decode_to_unicode(bytes),
        }
    }

    pub fn decode_chars(&self, bytes: &[u8]) -> Vec<CharCode> {
        match self {
            Font::Simple(sf) => sf.decode_chars(bytes),
            Font::Composite(cf) => cf.decode_chars(bytes),
        }
    }

    pub fn get_char_width(&self, charcode: &CharCode) -> f64 {
        match self {
            Font::Simple(sf) => sf.get_char_width(charcode),
            Font::Composite(cf) => cf.get_char_width(charcode),
        }
    }
    pub fn glyph_index_from_charcode(&self, charcode: &CharCode) -> Option<u32> {
        match self {
            Font::Simple(sf) => sf.glyph_index_from_charcode(charcode),
            Font::Composite(cf) => cf.glyph_index_from_charcode(charcode),
        }
    }

    pub fn get_glyph(&self, gid: u32, scale: u32) -> Option<GlyphSlot> {
        match self {
            Font::Simple(sf) => sf.get_glyph(gid, scale),
            Font::Composite(cf) => cf.get_glyph(gid, scale),
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Font::Simple(sf) => sf.basename(),
            Font::Composite(cf) => cf.basename(),
        }
    }
}

pub fn load_font<T: Seek + Read>(obj: &PDFObject, doc: &Document<T>) -> PDFResult<Font> {
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    match subtype.as_str() {
        "Type1" | "TrueType" => {
            let simple = load_simple_font(obj, doc)?;
            Ok(Font::Simple(simple))
        }
        "Type0" => {
            let type0 = load_composite_font(obj, doc)?;
            Ok(Font::Composite(type0))
        }
        _ => Err(PDFError::FontFailure(format!("subyte error:{:?}", obj))),
    }
}
