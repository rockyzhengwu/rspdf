use std::collections::HashMap;
use std::io::{Read, Seek};
use std::u32;

use freetype::{Bitmap, Face};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::font::cmap::predefined::get_predefine_cmap;
use crate::font::{cmap::CMap, load_face, Font};
use crate::object::{PDFArray, PDFObject};

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
) -> PDFResult<Font> {
    let mut widths = HashMap::new();
    let mut dw: u32 = 0;

    let mut face: Option<Face> = None;
    if let Some(descendant_fonts) = obj.get_value("DescendantFonts") {
        let descendant_fonts = doc.read_indirect(descendant_fonts)?;
        let df_ref = descendant_fonts.as_array()?.get(0).unwrap();
        let df_obj = doc.read_indirect(df_ref)?;
        dw = df_obj.get_value("DW").unwrap().as_f64()? as u32;
        if let Some(w_obj) = df_obj.get_value("W") {
            match w_obj {
                PDFObject::Arrray(arr) => widths = parse_widths(arr),
                PDFObject::Indirect(_) => {
                    let w_arr = doc.read_indirect(w_obj)?;
                    widths = parse_widths(w_arr.as_array()?);
                }
                _ => {
                    panic!("w need a array:{:?}", df_obj);
                }
            }
        }

        let df_desc = doc.read_indirect(df_obj.get_value("FontDescriptor").unwrap())?;
        let sstype = df_obj.get_value("Subtype").unwrap();
        let file = match sstype.as_string()?.as_str() {
            "CIDFontType0" => "FontFile3",
            _ => "FontFile2",
        };
        let font_file = doc.read_indirect(df_desc.get_value(file).unwrap())?;
        face = Some(load_face(font_file.bytes()?)?);
        //let mut file = std::fs::File::create("type0.otf").unwrap();
        //file.write_all(font_file.bytes()?.as_slice()).unwrap();
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
            }
            _ => {}
        }
    }
    // println!("encoding {:?}", encoding);

    let mut tounicode = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        tounicode = CMap::new_from_bytes(bytes.as_slice());
    }
    //println!("Tounicode {:?}", tounicode.code_to_character_len());
    unimplemented!()
}
