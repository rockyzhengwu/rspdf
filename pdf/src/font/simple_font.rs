use std::io::{Read, Seek};

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
    _fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<Font> {
    let mut font = Font::default();

    let basefont = obj.get_value_as_string("BaseFont").unwrap()?;
    println!("Dict {:?}", obj);

    font.widths = parse_widhts(obj)?;

    let mut face = None;
    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        font.descriptor = create_font_descriptor(&desc, &basefont)?;
        let ff = desc.get_value("FontFile");
        let ff2 = desc.get_value("FontFile2");
        let ff3 = desc.get_value("FontFile3");
        let program = ff.or(ff2).or(ff3);

        if let Some(emb) = program {
            let program = doc.read_indirect(emb)?;
            face = Some(load_face(program.bytes()?)?);
        } else {
            println!("{:?}", "builit");
            // TODO load load_builitin_font
        }
        println!("DESC {:?}", desc);
    } else {
        load_builitin_font(&basefont)?;
    }

    // TODO encoding
    if let Some(enc) = obj.get_value("Encoding") {
        let enc_obj = if enc.is_indirect() {
            doc.read_indirect(enc)?
        } else {
            enc.to_owned()
        };
        println!("enc:{:?}", enc_obj);
        // TODO is default encoding is None, select default encoding
        match enc_obj {
            PDFObject::Name(_) => {
                let name = enc_obj.as_string().unwrap();
                font.encoding = get_encoding(&name);
            }
            PDFObject::Dictionary(_) => {
                if let Some(base_enc) = enc_obj.get_value("BaseEncoding") {
                    let base_name = base_enc.as_string()?;
                    font.encoding = get_encoding(&base_name);
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
                            font.encoding.insert(code, name);
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
    for (code, name) in &font.encoding {
        if let Some(gid) = face.as_ref().unwrap().get_name_index(name) {
            font.cid_to_gid.insert(code.to_owned(), gid);
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
