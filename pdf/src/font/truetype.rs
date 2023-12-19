use std::collections::HashMap;
use std::io::{Read, Seek};

use freetype::{Bitmap, Face};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::font::cmap::CMap;
use crate::font::encoding::{predefine_encoding, FontEncoding};
use crate::font::load_face;
use crate::font::truetype_program::TrueTypeProgram;
use crate::object::{PDFObject, PDFString};

#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
pub struct TrueType {
    name: String,
    encoding: FontEncoding,
    tounicode: CMap,
    face: Option<Face>,
    program: Option<TrueTypeProgram>,
    widths: HashMap<u32, u32>,
}

impl TrueType {
    pub fn get_width(&self, code: &u32) -> u32 {
        self.widths.get(code).unwrap_or(&0_u32).to_owned()
    }

    pub fn get_unicode(&self, content: &PDFString) -> String {
        let bytes = content.binary_bytes();
        let cids: Vec<u32> = bytes.iter().map(|v| v.to_owned() as u32).collect();
        let s = self.tounicode.cid_to_string(cids.as_slice());
        s
    }

    pub fn decode_to_glyph(&self, code: u32, sx: u32, sy: u32) -> Bitmap {
        let gid = self.program.as_ref().unwrap().map_code_gid(code);
        match self.face {
            Some(ref f) => {
                f.set_pixel_sizes(sx, sy).unwrap();
                f.load_glyph(gid, freetype::face::LoadFlag::RENDER).unwrap();
                let glyph = f.glyph();
                glyph.bitmap()
            }
            None => {
                panic!("truee type doesn't hav face");
            }
        }
    }
}

pub fn create_truetype_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<TrueType> {
    let mut widths: HashMap<u32, u32> = HashMap::new();
    if let Some(ws) = obj.get_value("Widths") {
        let first_char = obj.get_value("FirstChar").unwrap().as_i64()?;
        let last_char = obj.get_value("LastChar").unwrap().as_i64()?;
        let ws = ws.as_array()?;
        for i in first_char..=last_char {
            widths.insert(
                (i & 0xffffffff) as u32,
                (ws[(i - first_char) as usize].as_i64().unwrap() & 0xffffffff) as u32,
            );
        }
    }

    let mut face: Option<Face> = None;
    let mut program = None;

    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        let font_file = desc.get_value("FontFile2").unwrap();
        let font_stream = doc.read_indirect(font_file)?;
        face = Some(load_face(font_stream.bytes()?)?);
        program = Some(TrueTypeProgram::new(font_stream.bytes()?));
    }

    // TODO encoding
    let mut encoding = FontEncoding::default();
    if let Some(enc) = obj.get_value("Encoding") {
        let enc_obj = if enc.is_indirect() {
            doc.read_indirect(enc)?
        } else {
            enc.to_owned()
        };
        match enc_obj {
            PDFObject::Name(name) => encoding = predefine_encoding(name.to_string().as_str()),
            PDFObject::Dictionary(_) => {}
            _ => {}
        }
        // TODO use diffs;
        //let _diffs = enc.get_value("Differences").unwrap();
    }

    let mut tounicode = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        tounicode = CMap::new_from_bytes(bytes.as_slice());
    }

    Ok(TrueType {
        name: fontname.to_string(),
        encoding,
        tounicode,
        face,
        program,
        widths,
    })
}
