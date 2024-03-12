use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::composite_font::{load_composite_font, CompositeFont};
use crate::font::simple_font::{load_simple_font, SimpleFont};
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub enum Font {
    Simple(SimpleFont),
    Composite(CompositeFont),
}

// TODO impl type0

impl Font {
    pub fn decode_to_cids(&self, bytes: &[u8]) -> Vec<u32> {
        match self {
            Font::Simple(sf) => sf.decode_to_cids(bytes),
            Font::Composite(cf) => cf.decode_to_cids(bytes),
        }
    }
    pub fn to_unicode(&self, bytes: &[u8]) -> Vec<String> {
        match self {
            Font::Simple(sf) => sf.decode_to_unicode(bytes),
            Font::Composite(cf) => cf.decode_to_unicode(bytes),
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
