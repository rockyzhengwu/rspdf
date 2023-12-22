use std::collections::HashMap;
use std::io::{Read, Seek};
use std::u32;

use freetype::{Bitmap, Face};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::font::cmap::predefined::get_predefine_cmap;
use crate::font::{cmap::CMap, load_face};
use crate::object::{PDFArray, PDFObject, PDFString};

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CompositeFont {
    name: String,
    encoding: CMap,
    tounicode: CMap,
    face: Option<Face>,
    widths: HashMap<u32, u32>,
    dw: u32,
}

impl CompositeFont {
    pub fn get_width(&self, code: &u32) -> u32 {
        match self.widths.get(code) {
            Some(w) => w.to_owned(),
            None => self.dw,
        }
    }

    pub fn get_unicode(&self, content: &PDFString) -> String {
        // bytes -> cid ;
        let bytes = content.binary_bytes();
        let cids = self.encoding.code_to_cid(bytes.as_slice());
        let s = self.tounicode.cid_to_string(cids.as_slice());
        s
    }

    pub fn decode_to_glyph(&self, code: u32, sx: u32, sy: u32) -> Bitmap {
        match self.face {
            Some(ref f) => {
                f.set_pixel_sizes(sx, sy).unwrap();
                f.load_char(code as usize, freetype::face::LoadFlag::RENDER)
                    .unwrap();
                let glyph = f.glyph();
                glyph.bitmap()
            }
            None => {
                panic!("face is None in font");
            }
        }
    }

    pub fn get_cids(&self, bytes: &[u8]) -> Vec<u32> {
        self.encoding.code_to_cid(bytes)
    }
}
fn parse_widths(w: &PDFArray) -> HashMap<u32, u32> {
    let mut widths = HashMap::new();
    let n = w.len();
    let mut i = 0;
    while i < n {
        let obj1 = w.get(i).unwrap().as_i64().unwrap() as u32;
        let obj2 = w.get(i + 1).unwrap();
        match obj2 {
            PDFObject::Arrray(arr) => {
                let mut start = obj1;
                for a in arr {
                    let aw = a.as_i64().unwrap() as u32;
                    widths.insert(start, aw);
                    start += 1;
                }
                i += 2;
            }
            PDFObject::Number(n) => {
                let aw = w.get(i + 2).unwrap().as_i64().unwrap() as u32;
                let end = n.as_i64() as u32;
                for k in obj1..end {
                    widths.insert(k, aw);
                }
                i += 3;
            }
            _ => {
                panic!("obj2 is need array or number")
            }
        }
    }
    widths
}

// TODO robust
pub fn create_composite_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<CompositeFont> {
    let mut widths = HashMap::new();
    let mut dw: u32 = 0;

    let mut face: Option<Face> = None;

    if let Some(descendant_fonts) = obj.get_value("DescendantFonts") {
        let descendant = doc.read_indirect(descendant_fonts)?;
        let desc_font = descendant.as_array()?.get(0).unwrap();
        let desc_font_obj = doc.read_indirect(desc_font)?;
        dw = desc_font_obj.get_value("DW").unwrap().as_f64()? as u32;
        match desc_font_obj.get_value("W") {
            Some(PDFObject::Arrray(arr)) => widths = parse_widths(arr),
            None => {}
            _ => {
                panic!("w need a array");
            }
        }

        let desc_font_descriptor =
            doc.read_indirect(desc_font_obj.get_value("FontDescriptor").unwrap())?;
        let sstype = desc_font_obj.get_value("Subtype").unwrap();
        let file = match sstype.as_string()?.as_str() {
            "CIDFontType0" => "FontFile3",
            _ => "FontFile2",
        };
        let font_file = doc.read_indirect(desc_font_descriptor.get_value(file).unwrap())?;
        face = Some(load_face(font_file.bytes()?)?);
    }

    let mut encoding = CMap::default();
    if let Some(enc) = obj.get_value("Encoding") {
        let enc_obj = if enc.is_indirect() {
            doc.read_indirect(enc)?
        } else {
            enc.to_owned()
        };
        match enc_obj {
            PDFObject::Name(name) => encoding = get_predefine_cmap(name.to_string().as_str()),
            PDFObject::Stream(s) => {
                let bytes = s.bytes();
                encoding = CMap::new_from_bytes(bytes.as_slice());
                println!("encoding")
            }
            _ => {}
        }
    }

    let mut tounicode = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        tounicode = CMap::new_from_bytes(bytes.as_slice());
    }
    //println!("Tounicode {:?}", tounicode.code_to_character_len());

    Ok(CompositeFont {
        name: fontname.to_string(),
        encoding,
        tounicode,
        face,
        widths,
        dw,
    })
}
