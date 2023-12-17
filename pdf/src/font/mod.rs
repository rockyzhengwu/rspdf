use std::io::{Read, Seek};

use freetype::{Bitmap, Face, Library};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFObject, PDFString};

pub(crate) mod cid_font;
pub(crate) mod cmap;
pub(crate) mod composite_font;
pub(crate) mod encoding;
pub(crate) mod simple_font;

use composite_font::{create_composite_font, CompositeFont};
use simple_font::{create_simple_font, SimpleFont};

#[derive(Clone, Debug, Default)]
pub enum Font {
    Simple(SimpleFont),
    Composite(CompositeFont),
    #[default]
    None,
}

impl Font {
    pub fn decode_to_glyph(&self, code: u32, sx: u32, sy: u32) -> Bitmap {
        match self {
            Font::Simple(sf) => sf.decode_to_glyph(code, sx, sy),
            Font::Composite(cf) => cf.decode_to_glyph(code, sx, sy),
            _ => panic!("not implemented"),
        }
    }

    pub fn get_unicode(&self, content: &PDFString) -> String {
        match self {
            Font::Simple(sf) => sf.get_unicode(content),
            Font::Composite(cf)=>cf.get_unicode(content),
            _ => panic!("unimplemented!"),
        }
    }

    pub fn get_width(&self, code: &u32) -> u32 {
        match self {
            Font::Simple(sf) => sf.get_width(code),
            Font::Composite(cf)=>cf.get_width(code),
            _ => panic!("unimplemented"),
        }
    }
}

pub fn create_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<Font> {
    println!("{:?},{:?}", fontname, obj);
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    match subtype.as_str() {
        "Type0" => Ok(Font::Composite(create_composite_font(fontname, obj, doc)?)),
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
