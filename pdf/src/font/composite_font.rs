use std::collections::HashMap;
use std::io::{Read, Seek};

use freetype::{Bitmap, Face};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::font::cid_font::CIDFont;
use crate::font::cmap::predefined::get_predefine_cmap;
use crate::font::{cmap::CMap, load_face};
use crate::object::{PDFObject, PDFString};

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CompositeFont {
    name: String,
    encoding: CMap,
    tounicode: CMap,
    cid: Option<CIDFont>,
    face: Option<Face>,
    widths: HashMap<u32, u32>,
}

impl CompositeFont {
    pub fn get_width(&self, code: &u32) -> u32 {
        self.widths.get(code).unwrap_or(&0_u32).to_owned()
    }

    pub fn get_unicode(&self, content: &PDFString) -> String {
        // bytes -> cid ;
        let bytes = content.binary_bytes();
        let cids = self.encoding.code_to_cid(bytes.as_slice());
        let s = self.tounicode.cid_to_string(cids.as_slice());
        println!(
            "unicode: {:?},{:?},{:?}, {:?},{}",
            s,
            cids,
            self.tounicode,
            content,
            self.encoding.name(),
        );
        s
    }

    pub fn decode_to_glyph(&self, _code: u32, _sx: u32, _sy: u32) -> Bitmap {
        unimplemented!()
    }
}

pub fn create_composite_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<CompositeFont> {
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
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
    let mut cid: Option<CIDFont> = None;

    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        let font_program = match subtype.as_str() {
            "TrueType" => "FontFile2",
            _ => "FontFile3",
        };
        let font_file = desc.get_value(font_program).unwrap();
        let font_stream = doc.read_indirect(font_file)?;
        face = Some(load_face(font_stream.bytes()?)?);
        if obj.get_value("Encoding").is_none() && subtype == "TrueType" {
            cid = Some(CIDFont::new(font_stream.bytes()?));
        }
    }

    // TODO encoding
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
        //let _diffs = enc.get_value("Differences").unwrap();
    }

    let mut tounicode = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        tounicode = CMap::new_from_bytes(bytes.as_slice());
    }

    Ok(CompositeFont {
        name: fontname.to_string(),
        encoding,
        tounicode,
        cid,
        face,
        widths,
    })
}
