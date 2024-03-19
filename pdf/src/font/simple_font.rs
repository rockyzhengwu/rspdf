use std::collections::HashMap;
use std::io::{Read, Seek};

use freetype::GlyphSlot;

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::cmap::charcode::CharCode;
use crate::font::cmap::predefined::get_predefine_cmap;
use crate::font::cmap::CMap;
use crate::font::encoding::{get_predefined_encoding, FontEncoding};
use crate::font::font_descriptor::FontDescriptor;
use crate::font::ft_font::FTFont;
use crate::font::truetype::load_truetype_glyph_map;
use crate::font::type1::load_typ1_glyph;
use crate::geom::rectangle::Rectangle;
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub struct SimpleFont {
    basename: String,
    desc: FontDescriptor,
    char_width: [i32; 256],
    glyph_index: [u32; 256],
    char_bbox: [Rectangle; 256],
    font_bbox: Rectangle,
    ft_font: FTFont,
    base_encoding: Option<FontEncoding>,
    diffs: HashMap<u8, String>,
    to_unicode: CMap,
}

impl Default for SimpleFont {
    fn default() -> Self {
        SimpleFont {
            basename: String::new(),
            desc: FontDescriptor::default(),
            char_width: [0; 256],
            glyph_index: [0; 256],
            char_bbox: [Rectangle::default(); 256],
            font_bbox: Rectangle::default(),
            ft_font: FTFont::default(),
            base_encoding: None,
            diffs: HashMap::new(),
            to_unicode: CMap::default(),
        }
    }
}

impl SimpleFont {
    pub fn basename(&self) -> &str {
        &self.basename
    }

    pub fn get_glyph(&self, gid: u32, scale: u32) -> Option<GlyphSlot> {
        self.ft_font.get_glyph(gid, scale)
    }

    pub fn glyph_index_from_charcode(&self, charcode: &CharCode) -> Option<u32> {
        self.glyph_index
            .get(charcode.code() as usize)
            .map(|x| x.to_owned())
    }

    pub fn get_char_width(&self, charcode: &CharCode) -> f64 {
        if charcode.code() > 256 {
            return 0.0;
        }
        let code = charcode.code() as u8;
        self.char_width[code as usize] as f64
    }

    pub fn decode_to_unicode(&self, bytes: &[u8]) -> Vec<String> {
        self.to_unicode.charcodes_to_unicode(bytes)
    }

    pub fn decode_chars(&self, bytes: &[u8]) -> Vec<CharCode> {
        bytes
            .iter()
            .map(|v| CharCode::new(v.to_owned() as u32, 1))
            .collect()
    }

    pub fn ft_font(&self) -> &FTFont {
        &self.ft_font
    }
    pub fn has_diffs(&self) -> bool {
        !self.diffs.is_empty()
    }

    pub fn set_glyph(&mut self, charcode: u8, glyph: u32) {
        // need charcode < 256
        self.glyph_index[charcode as usize] = glyph;
    }

    pub fn is_ttot(&self) -> bool {
        self.ft_font.is_ttot()
    }

    pub fn is_symbolic(&self) -> bool {
        self.desc.is_symbolic()
    }

    pub fn is_embeded(&self) -> bool {
        self.desc.is_embeded()
    }

    pub fn set_glyph_map_from_start(&mut self, startchar: u32) {
        if startchar > 256 {
            return;
        }
        for i in 0..startchar {
            self.glyph_index[i as usize] = 0;
        }
        let mut glyph_code: u32 = 3;
        for charcode in startchar..256 {
            self.glyph_index[charcode as usize] = glyph_code;
            glyph_code += 1;
        }
    }
    pub fn charname(&self, charcode: u8) -> Option<String> {
        if self.diffs.contains_key(&charcode) {
            return self.diffs.get(&charcode).map(|x| x.to_owned());
        }
        if let Some(encoding) = &self.base_encoding {
            return encoding.code_to_name(charcode).map(|x| x.to_owned());
        }
        None
    }

    pub fn base_encoding(&self) -> Option<&FontEncoding> {
        self.base_encoding.as_ref()
    }

    pub fn is_macrom_or_winasni(&self) -> bool {
        self.base_encoding == Some(FontEncoding::WinAnsi)
            || self.base_encoding == Some(FontEncoding::MacRoman)
    }
    pub fn has_glyph_names(&self) -> bool {
        self.ft_font.has_glyph_names()
    }
    pub fn num_charmaps(&self) -> i32 {
        self.ft_font.num_charmaps()
    }

    pub fn unicode_from_charcode(&self, charcode: u8) -> Option<u32> {
        if let Some(encoding) = &self.base_encoding {
            return encoding.unicode_from_charcode(charcode);
        }
        None
    }
}

pub enum CharmapType {
    MsUnicode,
    MsSymbol,
    MacRoman,
    Other,
}

fn load_width(font: &mut SimpleFont, obj: &PDFObject) -> PDFResult<()> {
    if let Some(widths) = obj.get_value("Widths") {
        let first_char = obj.get_value("FirstChar").unwrap().as_i32()?;
        let last_char = obj.get_value("LastChar").unwrap().as_i32()?;
        let ws = widths.as_array()?;
        if first_char > 255 {
            return Ok(());
        }
        for i in first_char..=last_char {
            let index = (i - first_char) as usize;
            if let Some(v) = ws.get(index) {
                font.char_width[i as usize] = v.as_i32()?;
            }
        }
    }
    Ok(())
}

fn load_encoding<T: Seek + Read>(
    obj: &PDFObject,
    font: &mut SimpleFont,
    doc: &Document<T>,
) -> PDFResult<()> {
    if !font.is_symbolic() {
        font.base_encoding = Some(FontEncoding::Standard);
    }
    if let Some(enc) = obj.get_value("Encoding") {
        let encoding = doc.get_object_without_indriect(enc)?;
        match encoding {
            PDFObject::Name(name) => {
                if font.is_symbolic() && font.basename == "Symbol" && !font.is_ttot() {
                    font.base_encoding = Some(FontEncoding::AdobeSymbol);
                } else {
                    font.base_encoding = get_predefined_encoding(name.name());
                }
            }
            PDFObject::Dictionary(dict) => {
                if let Some(name) = dict.get("BaseEncoding") {
                    font.base_encoding = get_predefined_encoding(&name.as_string()?);
                }

                if let Some(diff) = dict.get("Differences") {
                    let diffs = diff.as_array()?;
                    let mut diff_map = HashMap::new();
                    let mut code: usize = 0;
                    for df in diffs {
                        match df {
                            PDFObject::Number(n) => {
                                code = n.as_u32() as usize;
                            }
                            PDFObject::Name(_) => {
                                let name = df.as_string()?;
                                diff_map.insert(code as u8, name);
                                code += 1;
                            }
                            _ => {
                                return Err(PDFError::FontEncoding(format!(
                                    "encoding Differences need Name, or Number, got:{:?}",
                                    dict
                                )));
                            }
                        }
                    }
                    font.diffs = diff_map;
                }
            }
            _ => {}
        }
    }
    {
        if font.basename == "Symbol" {
            if font.is_ttot() {
                font.base_encoding = Some(FontEncoding::MsSymbol);
            } else {
                font.base_encoding = Some(FontEncoding::AdobeSymbol);
            }
        }
    }
    Ok(())
}

pub fn load_to_unicode(font: &mut SimpleFont, tu: &PDFObject) -> PDFResult<()> {
    match tu {
        PDFObject::Name(_) => {
            let name = tu.as_string()?;
            let cmap = get_predefine_cmap(&name);
            font.to_unicode = cmap;
        }
        PDFObject::Stream(_) => {
            let cmap = CMap::new_from_bytes(tu.bytes()?.as_slice())?;
            font.to_unicode = cmap;
        }
        _ => {
            // TODO set tounicode from encoding name
            for charcode in 0..=255 {
                // TODO add diffs fix this
                if let Some(unicode) = font.unicode_from_charcode(charcode) {
                    let s = char::from_u32(unicode).unwrap();
                    let mut ss = String::new();
                    ss.push(s);
                    font.to_unicode.add_simple_unicode(charcode as u32, ss);
                }
            }
        }
    }
    Ok(())
}

pub fn load_simple_font<T: Seek + Read>(
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<SimpleFont> {
    // TODO handle chinese font
    let mut font = SimpleFont::default();
    let subtype = obj.get_value_as_string("Subtype").unwrap()?;
    if let Some(s) = obj.get_value_as_string("BaseFont") {
        let name = s?;
        if name.find('+') == Some(6) {
            font.basename = name.split_once('+').unwrap().1.to_string();
        } else {
            font.basename = name;
        }
    }
    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.get_object_without_indriect(descriptor)?;
        font.desc = FontDescriptor::new_from_object(&desc)?;
    }
    if let Some(embeded) = font.desc.embeded() {
        let emb = doc.get_object_without_indriect(embeded)?;
        let ft_font = FTFont::try_new(emb.bytes()?)?;
        font.ft_font = ft_font;
    } else {
        font.ft_font = FTFont::try_new_builtin(&font.basename)?;
    }
    load_width(&mut font, obj)?;
    load_encoding(obj, &mut font, doc)?;
    match subtype.as_str() {
        "TrueType" => load_truetype_glyph_map(&mut font, obj)?,
        "Type1" => {
            load_typ1_glyph(&mut font)?;
        }
        _ => {} // TODO type0, type3
    }
    if let Some(tu) = obj.get_value("ToUnicode") {
        let tounicode = doc.get_object_without_indriect(tu)?;
        load_to_unicode(&mut font, &tounicode)?;
    }

    Ok(font)
}
