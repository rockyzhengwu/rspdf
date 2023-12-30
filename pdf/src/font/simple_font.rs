use std::collections::HashMap;
use std::io::{Read, Seek};

use log::warn;

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::builtin::load_builitin_font;
use crate::font::encoding::get_encoding;
use crate::font::{cmap::CMap, load_face, parse_widhts, Font, FontDescriptor};
use crate::object::PDFObject;

fn create_font_descriptor(desc: &PDFObject, _basefont: &str) -> PDFResult<FontDescriptor> {
    let mut d = FontDescriptor::default();
    if let Some(flags) = desc.get_value_as_i32("Flags") {
        d.flgs = flags?;
    }
    if let Some(ascent) = desc.get_value_as_f64("Ascent") {
        d.ascent = ascent?;
    }
    if let Some(cap_height) = desc.get_value_as_f64("CapHeight") {
        d.cap_height = cap_height?;
    }
    if let Some(x_height) = desc.get_value_as_f64("XHeight") {
        d.x_height = x_height?;
    }
    if let Some(descent) = desc.get_value_as_f64("Descent") {
        d.descent = descent?;
    }
    if let Some(missing_width) = desc.get_value_as_f64("MissingWidth") {
        d.missing_width = missing_width?;
    }
    if let Some(italic_angle) = desc.get_value_as_f64("ItalicAngle") {
        d.italic_angle = italic_angle?;
    }
    Ok(d)
}

pub fn create_simple_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<Font> {
    let mut font = Font::default();
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;

    let basefont = obj.get_value_as_string("BaseFont").unwrap()?;
    font.name = fontname.to_owned();
    font.widths = parse_widhts(obj)?;

    #[allow(unused_assignments)]
    let mut face = None;

    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        println!("{:?}", desc);
        font.descriptor = create_font_descriptor(&desc, &basefont)?;
        let ff = desc.get_value("FontFile");
        let ff2 = desc.get_value("FontFile2");
        let ff3 = desc.get_value("FontFile3");
        let program = ff.or(ff2).or(ff3);

        if let Some(emb) = program {
            let program = doc.read_indirect(emb)?;
            face = Some(load_face(program.bytes()?)?);
        } else {
            face = load_builitin_font(&basefont)?;
            // TODO load load_builitin_font
        }
    } else {
        face = load_builitin_font(&basefont)?;
    }
    // TODO fix, just load Helvetica as defalt
    // lookup font from system if not found math a smimilary defalut
    if face.is_none() {
        warn!("font not fount:{:?}", basefont);
        face = load_builitin_font("Helvetica")?;
    }

    // TODO FIX set cmap for face
    match subtype.as_str() {
        "TrueType" => {
            let num_charmap = face.as_ref().unwrap().num_charmaps();
            for i in 0..num_charmap {
                let charmap = face.as_ref().unwrap().get_charmap(i as isize);
                face.as_ref().unwrap().set_charmap(&charmap).unwrap();
            }
        }
        "Type1" => {}
        _ => {}
    }

    let mut encoding = HashMap::new();
    if let Some(enc) = obj.get_value("Encoding") {
        let enc_obj = if enc.is_indirect() {
            doc.read_indirect(enc)?
        } else {
            enc.to_owned()
        };
        // TODO is default encoding is None, select default encoding
        match enc_obj {
            PDFObject::Name(_) => {
                let name = enc_obj.as_string().unwrap();
                encoding = get_encoding(&name);
            }
            PDFObject::Dictionary(_) => {
                if let Some(base_enc) = enc_obj.get_value("BaseEncoding") {
                    let base_name = base_enc.as_string()?;
                    encoding = get_encoding(&base_name);
                }
                let difference = enc_obj.get_value("Differences").unwrap().as_array()?;
                let mut code: u32 = 0;
                for df in difference {
                    match df {
                        PDFObject::Number(n) => {
                            code = n.as_i64() as u32;
                        }
                        PDFObject::Name(_) => {
                            let name = df.as_string()?;
                            encoding.insert(code, name);
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
            }
            _ => {
                return Err(PDFError::FontEncoding(format!(
                    "encoding not a Name, or a Dictionary, got:{:?}",
                    enc_obj
                )));
            }
        }
    }
    if encoding.is_empty() {
        match obj.get_value_as_string("Subtype").unwrap()?.as_str() {
            "TrueType" => encoding = get_encoding("WinAnsiEncoding"),
            _ => panic!("not set default encoding"),
        };
    }

    for (code, name) in &encoding {
        if let Some(gid) = face.as_ref().unwrap().get_name_index(name) {
            font.cid_to_gid.insert(code.to_owned(), gid);
        }
    }

    let is_empty = font.cid_to_gid.is_empty();
    if is_empty {
        for i in 1..=256 {
            if let Some(gid) = face.as_ref().unwrap().get_char_index(i) {
                font.cid_to_gid.insert(i as u32, gid);
            } else if let Some(gd) = face.as_ref().unwrap().get_char_index(0xf000 + i) {
                font.cid_to_gid.insert(i as u32, gd);
            }
        }
    }

    let mut cmap = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        cmap = CMap::new_from_bytes(bytes.as_slice());
    }
    font.face = face;
    font.to_unicode = cmap;
    Ok(font)
}
