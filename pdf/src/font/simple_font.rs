use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::io::{Read, Seek};

use freetype::{face::LoadFlag, Bitmap, Face};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::encoding::{predefine_encoding, FontEncoding};
use crate::font::{cmap::CMap, load_face, parse_widhts};
use crate::object::{PDFObject, PDFString};

#[derive(Clone, Default)]
pub struct SimpleFont {
    #[allow(dead_code)]
    name: String,
    obj: PDFObject,
    tounicode: CMap,
    widths: HashMap<u32, u32>,
    face: Option<Face>,
    code_to_gid: Option<HashMap<u32, u32>>,
    font_encoding: Option<FontEncoding>,
}
impl SimpleFont {
    pub fn new(
        name: &str,
        obj: PDFObject,
        tounicode: CMap,
        widths: HashMap<u32, u32>,
        face: Option<Face>,
        code_to_gid: Option<HashMap<u32, u32>>,
        font_encoding: Option<FontEncoding>,
    ) -> Self {
        SimpleFont {
            name: name.to_string(),
            obj,
            tounicode,
            widths,
            face,
            code_to_gid,
            font_encoding,
        }
    }

    pub fn get_cids(&self, bytes: &[u8]) -> Vec<u32> {
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
                if self.code_to_gid.is_some() {
                    //println!("glyph: {:?},{:?}, {:?}", code, self.code_to_gid, self.name);
                    let gid = if self.code_to_gid.is_some() {
                        self.code_to_gid.as_ref().unwrap().get(&code).unwrap()
                    } else {
                        self.font_encoding
                            .as_ref()
                            .unwrap()
                            .code_to_cid(code)
                            .unwrap()
                    };
                    f.load_glyph(gid.to_owned(), LoadFlag::RENDER).unwrap();
                } else {
                    f.load_char(code as usize, LoadFlag::RENDER).unwrap();
                }
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
    let widths = parse_widhts(obj)?;

    let mut face: Option<Face> = None;

    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        let font_program = match subtype.as_str() {
            "TrueType" => "FontFile2",
            "Type1" => {
                if desc.get_value("FontFile").is_some() {
                    // type1
                    "FontFile"
                } else {
                    // Type1C
                    "FontFile3"
                }
            }
            _ => "FontFile3",
        };
        //println!("Fontdescriptor {:?}", desc);
        let font_file = desc.get_value(font_program).unwrap();
        let font_stream = doc.read_indirect(font_file)?;
        face = Some(load_face(font_stream.bytes()?)?);
        //let lengths = font_stream.get_value("Length1").unwrap();
    }

    // TODO encoding
    let mut code_to_gid = None;
    let mut font_encoding = None;
    if let Some(enc) = obj.get_value("Encoding") {
        let enc_obj = if enc.is_indirect() {
            doc.read_indirect(enc)?
        } else {
            enc.to_owned()
        };
        match enc_obj {
            PDFObject::Name(_) => {
                font_encoding = Some(predefine_encoding(enc_obj.as_string().unwrap().as_str()))
            }
            PDFObject::Dictionary(_) => {
                let difference = enc_obj.get_value("Differences").unwrap().as_array()?;
                let mut to_gid = HashMap::new();
                let mut code: u32 = 0;
                for df in difference {
                    match df {
                        PDFObject::Number(n) => {
                            code = n.as_i64() as u32;
                        }
                        PDFObject::Name(_) => {
                            let name = df.as_string()?;
                            let gid = face
                                .as_ref()
                                .unwrap()
                                .get_name_index(name.as_str())
                                .unwrap();
                            to_gid.insert(code, gid);
                            code += 1;
                        }
                        _ => {
                            return Err(PDFError::FontEncoding(format!(
                                "encoding Differences need Name, or Number, got:{:?}",
                                enc_obj
                            )));
                        }
                    }
                }
                code_to_gid = Some(to_gid);
            }
            _ => {
                return Err(PDFError::FontEncoding(format!(
                    "encoding not a Name, or a Dictionary, got:{:?}",
                    enc_obj
                )));
            }
        }
    }

    let mut cmap = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        cmap = CMap::new_from_bytes(bytes.as_slice());
    }

    Ok(SimpleFont::new(
        fontname,
        obj.to_owned(),
        cmap,
        widths,
        face,
        code_to_gid,
        font_encoding,
    ))
}
