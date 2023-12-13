use std::collections::HashMap;
use std::io::{Read, Seek};

use freetype::{Face, Library};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::cid_font::CIDFont;
use crate::font::cmap::CMap;
use crate::object::PDFObject;

pub(crate) mod cid_font;
mod cmap;
pub(crate) mod simple_font;

use simple_font::PDFFont;

pub fn create_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<PDFFont> {
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    let mut width_map: HashMap<u32, u32> = HashMap::new();
    if let Some(widths) = obj.get_value("Widths") {
        let first_char = obj.get_value("FirstChar").unwrap().as_i64()?;
        let last_char = obj.get_value("LastChar").unwrap().as_i64()?;
        let ws = widths.as_array()?;
        for i in first_char..=last_char {
            // TODO handle this
            width_map.insert(
                (i & 0xffffffff) as u32,
                (ws[(i - first_char) as usize].as_i64().unwrap() & 0xffffffff) as u32,
            );
        }
    }

    let mut face: Option<Face> = None;
    let mut cid: Option<CIDFont> = None;

    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        let font_program = match subtype.as_str() {
            "TrueType" => "FontFile2",
            _ => "FontFile3",
        };
        let font_file = desc.get_value(font_program).unwrap();
        let font_stream = doc.read_indirect(font_file)?;
        //let lengths = font_stream.get_value("Length1").unwrap();

        face = Some(load_face(font_stream.bytes()?)?);
        if obj.get_value("Encoding").is_none() && subtype == "TrueType" {
            cid = Some(CIDFont::new(font_stream.bytes()?));
        }
    }

    // TODO encoding
    if let Some(_enc) = obj.get_value("Encoding") {
        //let enc = doc.read_indirect(enc)?;
        //let _diffs = enc.get_value("Differences").unwrap();
    }

    let mut cmap = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        cmap = CMap::new(bytes.as_slice());
        // parse cmap
    }
    Ok(PDFFont::new(
        fontname,
        obj.to_owned(),
        cmap,
        width_map,
        face,
        cid,
    ))
}

fn load_face(buffer: Vec<u8>) -> PDFResult<Face> {
    let lib = Library::init().unwrap();
    match lib.new_memory_face(buffer, 0) {
        Ok(face) => Ok(face),
        Err(e) => Err(PDFError::FontFreeType(format!("Load face error{:?}", e))),
    }
}
