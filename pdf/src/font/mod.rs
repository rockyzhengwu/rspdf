use std::collections::HashMap;
use std::io::{Read, Seek};

use freetype::GlyphSlot;
use freetype::{face::LoadFlag, Face, Library};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::charinfo::CharInfo;
use crate::font::cmap::CMap;
use crate::geom::rectangle::Rectangle;
use crate::object::PDFObject;

pub(crate) mod builtin;
pub(crate) mod charinfo;
pub(crate) mod cmap;
pub(crate) mod composite_font;
pub(crate) mod encoding;
pub(crate) mod glyph_name;
pub(crate) mod simple_font;

use composite_font::create_composite_font;
use simple_font::create_simple_font;

#[derive(Clone, Debug, Default)]
pub struct FontDescriptor {
    flgs: u32,
    italic_angle: f64,
    ascent: f64,
    descent: f64,
    cap_height: f64,
    x_height: f64,
    missing_width: f64,
    stem_v: f64,
    bbox: Rectangle,
}

impl FontDescriptor {
    pub fn is_symbolic(&self) -> bool {
        self.flgs & 4 == 0
    }
}

#[derive(Clone, Debug, Default)]
pub struct Font {
    name: String,
    base_name: String,
    subtype: String,
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
    pub fn face(&self) -> &Face {
        self.face.as_ref().unwrap()
    }

    pub fn is_symbolic(&self) -> bool {
        self.descriptor.is_symbolic()
    }

    pub fn is_truetype(&self) -> bool {
        self.subtype == "TrueType"
    }

    pub fn set_face(&mut self, face: Option<Face>) {
        self.face = face;
    }

    pub fn get_glyph(&self, code: &u32, scale: &u32) -> Option<GlyphSlot> {
        match self.face {
            Some(ref f) => {
                f.set_pixel_sizes(scale.to_owned(), scale.to_owned())
                    .unwrap();
                let gid = {
                    if !self.cid_to_gid.is_empty() {
                        self.cid_to_gid.get(code).unwrap().to_owned()
                    } else {
                        code.to_owned()
                    }
                };
                // TODO todo return Error install of unwrap
                f.load_glyph(gid, LoadFlag::RENDER).unwrap();
                let glyph = f.glyph();
                Some(glyph.to_owned())
            }
            None => {
                panic!("face is None in font");
            }
        }
    }

    // TODO handle mutil bytes to one char
    pub fn decode_charcodes(&self, bytes: &[u8]) -> Vec<CharInfo> {
        let mut res = Vec::new();
        let mut len: u8 = 1;
        if let Some(enc) = &self.encoding {
            if enc.name() == "Identity-H" || enc.name() == "Identity-V" {
                len = 2;
            }
        }
        let mut p = 0;
        while p < bytes.len() {
            let mut code: u32 = 0;
            for i in 0..len {
                code += bytes[p + i as usize] as u32;
            }
            let cid = match self.encoding {
                Some(ref enc) => enc.charcode_to_cid(&code),
                None => code,
            };
            p += len as usize;
            let unicode = self.to_unicode.cid_to_unicode(&code);
            let char = CharInfo::new(len, code, cid, unicode);
            res.push(char);
        }
        res
    }

    pub fn get_unicode(&self, cids: &[u32]) -> String {
        self.to_unicode.decode_string(cids)
    }

    pub fn cid_to_unicode(&self, cid: &u32) -> char {
        self.to_unicode.cid_to_unicode(cid)
    }

    pub fn charcode_to_unicode(&self, bytes: &[u8]) -> Vec<char> {
        let mut res = Vec::new();
        for code in bytes {
            let c = self.to_unicode.cid_to_unicode(&(code.to_owned() as u32));
            res.push(c);
        }
        res
    }

    pub fn get_width(&self, code: &u32) -> f64 {
        match self.widths.get(code) {
            Some(w) => w.to_owned(),
            None => self.dwidths,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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
            panic!("font type didn't support yet:{:?}", subtype);
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
