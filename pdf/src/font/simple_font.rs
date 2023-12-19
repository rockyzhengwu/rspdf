use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::io::{Read, Seek};

use freetype::{Bitmap, Face};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::font::{cmap::CMap, load_face};
use crate::object::{PDFObject, PDFString};

#[derive(Clone, Default)]
pub struct SimpleFont {
    #[allow(dead_code)]
    name: String,
    obj: PDFObject,
    tounicode: CMap,
    widths: HashMap<u32, u32>,
    face: Option<Face>,
}
impl SimpleFont {
    pub fn new(
        name: &str,
        obj: PDFObject,
        tounicode: CMap,
        widths: HashMap<u32, u32>,
        face: Option<Face>,
    ) -> Self {
        SimpleFont {
            name: name.to_string(),
            obj,
            tounicode,
            widths,
            face,
        }
    }

    pub fn get_gids(&self, bytes: &[u8]) -> Vec<u32> {
        let mut res: Vec<u32> = Vec::new();
        for code in bytes {
            res.push(code.to_owned() as u32);
        }
        res
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

    pub fn get_unicode(&self, content: &PDFString) -> String {
        if self.tounicode.has_to_unicode() {
            return content.to_string();
        }
        self.tounicode.decode_string(content)
    }

    pub fn get_width(&self, code: &u32) -> u32 {
        self.widths.get(code).unwrap_or(&0_u32).to_owned()
    }
}

impl Debug for SimpleFont {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PDFFont")
            .field("font_dict:", &self.obj)
            .finish()
    }
}

pub fn create_simple_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<SimpleFont> {
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    let mut width_map: HashMap<u32, u32> = HashMap::new();
    if let Some(widths) = obj.get_value("Widths") {
        let first_char = obj.get_value("FirstChar").unwrap().as_i64()?;
        let last_char = obj.get_value("LastChar").unwrap().as_i64()?;
        let ws = widths.as_array()?;
        for i in first_char..=last_char {
            width_map.insert(
                (i & 0xffffffff) as u32,
                (ws[(i - first_char) as usize].as_i64().unwrap() & 0xffffffff) as u32,
            );
        }
    }

    let mut face: Option<Face> = None;

    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        let font_program = match subtype.as_str() {
            "TrueType" => "FontFile2",
            _ => "FontFile3",
        };
        let font_file = desc.get_value(font_program).unwrap();
        let font_stream = doc.read_indirect(font_file)?;
        face = Some(load_face(font_stream.bytes()?)?);
        //let lengths = font_stream.get_value("Length1").unwrap();
    }

    // TODO encoding
    if let Some(enc) = obj.get_value("Encoding") {
        let enc_obj = if enc.is_indirect() {
            doc.read_indirect(enc)?
        } else {
            enc.to_owned()
        };
        println!("encoding {:?}", enc_obj);
        //let _diffs = enc.get_value("Differences").unwrap();
    }

    let mut cmap = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        //println!("ToUnicode {:?}", String::from_utf8_lossy(bytes.as_slice()));
        cmap = CMap::new_from_bytes(bytes.as_slice());
    }

    Ok(SimpleFont::new(
        fontname,
        obj.to_owned(),
        cmap,
        width_map,
        face,
    ))
}
