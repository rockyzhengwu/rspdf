use std::collections::HashMap;
use std::io::{Read, Seek};

use freetype::{face::LoadFlag, Bitmap, Face, Library};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::cmap::CMap;
use crate::object::{PDFObject, PDFString};

pub(crate) mod builtin;
pub(crate) mod cmap;
pub(crate) mod composite_font;
pub(crate) mod encoding;
pub(crate) mod simple_font;
pub(crate) mod truetype_program;

use composite_font::create_composite_font;
use simple_font::create_simple_font;

#[derive(Clone, Debug, Default)]
pub struct FontDescriptor {
    flgs: i32,
    italic_angle: f64,
    ascent: f64,
    descent: f64,
    cap_height: f64,
    x_height: f64,
    missing_width: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Font {
    name: String,
    //
    descriptor: FontDescriptor,

    encoding: Option<CMap>,
    cid_to_gid: HashMap<u32, u32>,
    to_unicode: CMap,

    widths: HashMap<u32, f64>,
    dwidths: f64,
    face: Option<Face>,
}

impl Font {
    pub fn decode_to_glyph(&self, code: u32, sx: u32, sy: u32) -> Option<Bitmap> {
        match self.face {
            Some(ref f) => {
                f.set_pixel_sizes(sx, sy).unwrap();
                let gid = {
                    if !self.cid_to_gid.is_empty() {
                        self.cid_to_gid.get(&code).unwrap().to_owned()
                    } else {
                        code
                    }
                };
                f.load_glyph(gid, LoadFlag::RENDER).unwrap();
                let glyph = f.glyph();
                Some(glyph.bitmap())
            }
            None => {
                panic!("face is None in font");
            }
        }
    }

    // TODO impl this
    pub fn code_to_gids(&self, bytes: &[u8]) -> Vec<u32> {
        let mut res = Vec::new();
        if let Some(enc) = &self.encoding {
            res = enc.code_to_gid(bytes);
        } else {
            for code in bytes {
                res.push(code.to_owned() as u32);
            }
        }
        res
    }

    pub fn get_unicode(&self, content: &PDFString) -> String {
        //let gids = self.code_to_gids(content.binary_bytes().as_slice());
        if let Some(enc) = &self.encoding {
            let cids = enc.code_to_gid(content.bytes());
            self.to_unicode.decode_string(cids.as_slice())
        } else {
            let mut gids = Vec::new();
            for code in content.binary_bytes() {
                gids.push(code as u32);
            }
            self.to_unicode.decode_string(gids.as_slice())
        }
    }

    pub fn get_width(&self, code: &u32) -> f64 {
        match self.widths.get(code) {
            Some(w) => w.to_owned(),
            None => self.dwidths,
        }
    }
}

pub fn create_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<Font> {
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    match subtype.as_str() {
        "Type0" => create_composite_font(fontname, obj, doc),
        "Type1" | "TrueType" => create_simple_font(fontname, obj, doc),
        _ => {
            panic!("didn't implement");
        }
    }
}

fn load_face(buffer: Vec<u8>) -> PDFResult<Face> {
    let lib = Library::init().unwrap();
    match lib.new_memory_face(buffer, 0) {
        Ok(face) => Ok(face),
        Err(e) => Err(PDFError::FontFreeType(format!("Load face error{:?}", e))),
    }
}

pub fn parse_widhts(obj: &PDFObject) -> PDFResult<HashMap<u32, f64>> {
    let mut width_map: HashMap<u32, f64> = HashMap::new();
    if let Some(widths) = obj.get_value("Widths") {
        let first_char = obj.get_value("FirstChar").unwrap().as_i64()?;
        let last_char = obj.get_value("LastChar").unwrap().as_i64()?;
        let ws = widths.as_array()?;
        for i in first_char..=last_char {
            width_map.insert(
                (i & 0xffffffff) as u32,
                ws[(i - first_char) as usize].as_f64().unwrap(),
            );
        }
    }
    Ok(width_map)
}
