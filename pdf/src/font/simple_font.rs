use std::collections::HashMap;
use std::io::{Read, Seek};

use log::warn;

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::builtin::load_builitin_font;
use crate::font::encoding::get_encoding;
use crate::font::glyph_name::name_to_unicode;
use crate::font::{cmap::CMap, load_face, parse_widhts, Font, FontDescriptor};
use crate::geom::rectangle::Rectangle;
use crate::object::PDFObject;

fn create_font_descriptor<T: Seek + Read>(
    desc: &PDFObject,
    basefont: &str,
    font: &mut Font,
    doc: &Document<T>,
) -> PDFResult<()> {
    let mut d = FontDescriptor::default();
    if let Some(flags) = desc.get_value_as_u32("Flags") {
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
    if let Some(stem_v) = desc.get_value_as_f64("StemV") {
        d.stem_v = stem_v?;
    }
    if let Some(PDFObject::Arrray(values)) = desc.get_value("FontBBox") {
        let lx = values[0].as_f64()?;
        let ly = values[1].as_f64()?;
        let ux = values[2].as_f64()?;
        let uy = values[3].as_f64()?;
        let rectangle = Rectangle::new(lx, ly, ux, uy);
        d.bbox = rectangle;
    }

    font.descriptor = d;
    let ff = desc.get_value("FontFile");
    let ff2 = desc.get_value("FontFile2");
    let ff3 = desc.get_value("FontFile3");
    let program = ff.or(ff2).or(ff3);
    let face = {
        if let Some(emb) = program {
            let program = doc.read_indirect(emb)?;
            Some(load_face(program.bytes()?)?)
        } else {
            load_builitin_font(basefont)?
        }
    };
    font.face = face;
    Ok(())
}

fn parse_basefont(name: String) -> String {
    if name.len() > 7 && name.chars().nth(6) == Some('+') {
        let (_, last) = name.split_at(7);
        last.to_string()
    } else {
        name
    }
}

pub fn set_charmap(font: &mut Font, subtype: &str) {
    match subtype {
        "TrueType" => {
            // ISO3200 9.6.6.4
            let mut setted = false;
            let num_charmap = font.face.as_ref().unwrap().num_charmaps();
            if font.is_symbolic() {
                for i in 0..num_charmap {
                    let charmap = font.face().get_charmap(i as isize);
                    if charmap.platform_id() == 3 && charmap.encoding_id() == 0 {
                        setted = true;
                        font.face().set_charmap(&charmap).unwrap();
                    }
                }
            }
            if !setted {
                for i in 0..num_charmap {
                    let charmap = font.face().get_charmap(i as isize);
                    if charmap.platform_id() == 3 && charmap.encoding_id() == 1 {
                        setted = true;
                        font.face().set_charmap(&charmap).unwrap();
                    }
                }
            }
            if !setted {
                for i in 0..num_charmap {
                    let charmap = font.face().get_charmap(i as isize);
                    if charmap.platform_id() == 1 && charmap.encoding_id() == 0 {
                        setted = true;
                        font.face().set_charmap(&charmap).unwrap();
                    }
                }
            }
            if !setted && num_charmap > 0 {
                font.face()
                    .set_charmap(&font.face().get_charmap(0))
                    .unwrap();
            }
        }
        "Type1" => {
            let num_charmap = font.face.as_ref().unwrap().num_charmaps();
            let mut founded = false;
            for i in 0..num_charmap {
                let charmap = font.face().get_charmap(i as isize);
                if charmap.platform_id() == 7 {
                    founded = true;
                    font.face().set_charmap(&charmap).unwrap();
                }
            }

            if !founded && num_charmap > 0 {
                font.face()
                    .set_charmap(&font.face().get_charmap(0))
                    .unwrap();
            }
        }
        _ => {}
    }
}

fn create_encoding<T: Seek + Read>(
    font: &mut Font,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<()> {
    let mut encoding = HashMap::new();
    match obj.get_value("Encoding") {
        Some(enc) => {
            let enc_obj = if enc.is_indirect() {
                doc.read_indirect(enc)?
            } else {
                enc.to_owned()
            };

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
        None => {
            if font.base_name == "Symbol" || font.is_symbolic() {
                encoding = get_encoding("Symbol");
            } else {
                encoding = get_encoding("WinAnsiEncoding");
            }
        }
    }

    for (code, name) in &encoding {
        if let Some(gid) = font.face().get_name_index(name) {
            font.cid_to_gid.insert(code.to_owned(), gid);
        }
    }

    let mut cmap = CMap::default();
    if let Some(tu) = obj.get_value("ToUnicode") {
        let to_unicode = doc.read_indirect(tu)?;
        let bytes = to_unicode.bytes()?;
        cmap = CMap::new_from_bytes(bytes.as_slice())?;
    } else {
        for (code, name) in &encoding {
            if name.is_empty() {
                continue;
            }
            if let Some(u) = name_to_unicode(name) {
                cmap.add_character(code.to_owned(), u);
            } else {
                warn!("not found character:{:?},{:?}", code, name);
            }
        }
    }

    font.to_unicode = cmap;

    Ok(())
}

pub fn create_simple_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<Font> {
    let mut font = Font::default();
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    font.subtype = subtype.to_string();

    let basefont = match obj.get_value_as_string("BaseFont") {
        Some(name) => parse_basefont(name?),
        _ => "".to_string(),
    };

    font.name = fontname.to_owned();
    font.widths = parse_widhts(obj)?;
    font.base_name = basefont.clone();

    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.read_indirect(descriptor)?;
        create_font_descriptor(&desc, &basefont, &mut font, doc)?;
    } else {
        font.face = load_builitin_font(&basefont)?;
    }
    // TODO fix, just load Helvetica as defalt
    // lookup font from system if not found math a smimilary defalut
    if font.face.is_none() {
        warn!("font not fount:{:?}", basefont);
        font.face = load_builitin_font("Helvetica")?;
    }

    set_charmap(&mut font, subtype.as_str());
    create_encoding(&mut font, obj, doc)?;

    // TODO FIX set cmap for face
    let is_empty = font.cid_to_gid.is_empty();
    if is_empty {
        for i in 1..=256 {
            if let Some(gid) = font.face().get_char_index(i) {
                font.cid_to_gid.insert(i as u32, gid);
            } else if let Some(gd) = font.face().get_char_index(0xf000 + i) {
                font.cid_to_gid.insert(i as u32, gd);
            }
        }
    }

    Ok(font)
}
