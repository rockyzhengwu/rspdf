use std::io::{Read, Seek};

use freetype::{Bitmap, Face, Library};
use log::warn;

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::truetype::create_truetype_font;
use crate::object::{PDFObject, PDFString};

pub(crate) mod cmap;
pub(crate) mod composite_font;
pub(crate) mod encoding;
pub(crate) mod simple_font;
pub(crate) mod truetype;
pub(crate) mod truetype_program;

use composite_font::{create_composite_font, CompositeFont};
use simple_font::{create_simple_font, SimpleFont};
use truetype::TrueType;

#[derive(Clone, Debug, Default)]
pub enum Font {
    Simple(SimpleFont),
    Composite(CompositeFont),
    TrueType(TrueType),
    #[default]
    None,
}

impl Font {
    pub fn decode_to_glyph(&self, code: u32, sx: u32, sy: u32) -> Bitmap {
        match self {
            Font::Simple(sf) => sf.decode_to_glyph(code, sx, sy),
            Font::Composite(cf) => cf.decode_to_glyph(code, sx, sy),
            Font::TrueType(tf) => tf.decode_to_glyph(code, sx, sy),
            _ => panic!("not implemented"),
        }
    }

    pub fn get_unicode(&self, content: &PDFString) -> String {
        match self {
            Font::Simple(sf) => sf.get_unicode(content),
            Font::Composite(cf) => cf.get_unicode(content),
            Font::TrueType(tf) => tf.get_unicode(content),
            _ => {
                warn!("not supported font:{:?}", self);
                "".to_string()
            }
        }
    }

    pub fn get_width(&self, code: &u32) -> u32 {
        //println!("{:?}", self);
        match self {
            Font::Simple(sf) => sf.get_width(code),
            Font::Composite(cf) => cf.get_width(code),
            Font::TrueType(tf) => tf.get_width(code),
            _ => {
                warn!("not support font:{:?}", self);
                0
            }
        }
    }
}

pub fn create_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<Font> {
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    println!("{:?},{:?}", subtype, fontname);
    match subtype.as_str() {
        "Type0" => Ok(Font::Composite(create_composite_font(fontname, obj, doc)?)),
        "TrueType" => Ok(Font::TrueType(create_truetype_font(fontname, obj, doc)?)),
        _ => Ok(Font::Simple(create_simple_font(fontname, obj, doc)?)),
    }
}

fn load_face(buffer: Vec<u8>) -> PDFResult<Face> {
    let lib = Library::init().unwrap();
    match lib.new_memory_face(buffer, 0) {
        Ok(face) => Ok(face),
        Err(e) => Err(PDFError::FontFreeType(format!("Load face error{:?}", e))),
    }
}
