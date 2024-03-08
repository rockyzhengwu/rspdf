use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::encoding::{
    char_name_from_predefined_encoding, get_predefined_encoding, FontEncoding,
};
use crate::font::ft_font::FTFont;
use crate::font::to_unicode::ToUnicodeMap;
use crate::geom::rectangle::Rectangle;
use crate::object::PDFObject;

pub struct TrueTypeFont {
    basename: String,
    flags: i32,
    stemv: i32,
    ascent: i32,
    descent: i32,
    italic_angle: i32,
    char_width: [i32; 256],
    glphy_index: [u32; 256],
    char_bbox: [Rectangle; 256],
    font_bbox: Rectangle,
    ft_font: FTFont,
    base_encoding: Option<FontEncoding>,
    diffs: HashMap<u8, String>,
    is_embed: bool,
}
pub enum CharmapType {
    MsUnicode,
    MsSymbol,
    MacRoman,
    Other,
}

impl TrueTypeFont {
    pub fn is_ttot(&self) -> bool {
        self.ft_font.is_ttot()
    }

    pub fn is_style_symbolic(&self) -> bool {
        (self.flags & 4) != 0
    }

    pub fn determain_encoding(&self) -> Option<FontEncoding> {
        if !self.is_embed
            && !self.is_style_symbolic()
            && !matches!(
                self.base_encoding,
                Some(FontEncoding::WinAnsi) | Some(FontEncoding::MacRoman)
            )
        {
            return self.base_encoding.to_owned();
        }
        let num_charmaps = self.ft_font.num_charmaps();
        if num_charmaps == 0 {
            return self.base_encoding.to_owned();
        }
        let mut support_mac = false;
        let mut support_win = false;
        for charmap_id in 0..num_charmaps {
            if let Some(charmap) = self.ft_font.charmap(charmap_id) {
                let platform_id = charmap.platform_id();
                match platform_id {
                    0 | 3 => {
                        support_win = true;
                    }
                    1 => {
                        support_mac = true;
                    }
                    _ => {}
                }
                if support_win && support_mac {
                    break;
                }
            }
        }
        match self.base_encoding {
            Some(FontEncoding::WinAnsi) => {
                if !support_win && support_mac {
                    return Some(FontEncoding::MacRoman);
                }
                return None;
            }
            Some(FontEncoding::MacRoman) => {
                if !support_mac && support_win {
                    return Some(FontEncoding::WinAnsi);
                }
                return None;
            }
            _ => {}
        }
        self.base_encoding.to_owned()
    }

    pub fn determain_charmap_type(&self) -> PDFResult<CharmapType> {
        if self.ft_font.use_charmaps_ms_unicode() {
            return Ok(CharmapType::MsUnicode);
        }
        if !self.is_style_symbolic() {
            if self.ft_font.use_charmaps_mac_rom() {
                return Ok(CharmapType::MacRoman);
            }
            if self.ft_font.use_charmaps_ms_symbol() {
                return Ok(CharmapType::MsSymbol);
            }
        } else {
            if self.ft_font.use_charmaps_ms_symbol() {
                return Ok(CharmapType::MsSymbol);
            }

            if self.ft_font.use_charmaps_mac_rom() {
                return Ok(CharmapType::MacRoman);
            }
        }
        Ok(CharmapType::Other)
    }

    pub fn set_glyph_map_from_start(&mut self, startchar: u32) {
        if startchar > 256 {
            return;
        }
        for i in 0..startchar {
            self.glphy_index[i as usize] = 0;
        }
        let mut glyph_code: u32 = 3;
        for charcode in startchar..256 {
            self.glphy_index[charcode as usize] = glyph_code;
            glyph_code += 1;
        }
    }
    pub fn charname(&self, charcode: u8, encoding: &FontEncoding) -> Option<String> {
        if self.diffs.contains_key(&charcode) {
            return self.diffs.get(&charcode).map(|x| x.to_owned());
        }
        char_name_from_predefined_encoding(encoding, charcode).map(|x| x.to_owned())
    }
}

impl Default for TrueTypeFont {
    fn default() -> Self {
        TrueTypeFont {
            basename: String::new(),
            flags: 0,
            stemv: 0,
            ascent: 0,
            descent: 0,
            italic_angle: 0,
            char_width: [0; 256],
            glphy_index: [0; 256],
            char_bbox: [Rectangle::default(); 256],
            font_bbox: Rectangle::default(),
            ft_font: FTFont::default(),
            base_encoding: None,
            diffs: HashMap::new(),
            is_embed: false,
        }
    }
}

fn load_font_descriptor(desc: &PDFObject, font: &mut TrueTypeFont) -> PDFResult<()> {
    if let Some(flags) = desc.get_value_as_i32("Flags") {
        font.flags = flags?;
    }
    if let Some(italic_angle) = desc.get_value_as_i32("ItalicAngle") {
        font.italic_angle = italic_angle?;
    }
    if let Some(stemv) = desc.get_value_as_i32("StemV") {
        font.stemv = stemv?;
    }

    if let Some(ascent) = desc.get_value_as_i32("Ascent") {
        font.ascent = ascent?;
    }
    if let Some(descent) = desc.get_value_as_i32("Descent") {
        font.descent = descent?;
    }

    if let Some(PDFObject::Arrray(values)) = desc.get_value("FontBBox") {
        let lx = values[0].as_f64()?;
        let ly = values[1].as_f64()?;
        let ux = values[2].as_f64()?;
        let uy = values[3].as_f64()?;
        let rectangle = Rectangle::new(lx, ly, ux, uy);
        font.font_bbox = rectangle;
    }
    if let Some(missing_width) = desc.get_value_as_i32("MissingWidth") {
        let width = missing_width?;
        font.char_width = [width; 256];
    }
    Ok(())
}

fn load_width(font: &mut TrueTypeFont, obj: &PDFObject) -> PDFResult<()> {
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
        return Ok(());
    }
    Ok(())
}

fn load_encoding(obj: &PDFObject, font: &mut TrueTypeFont) -> PDFResult<()> {
    let encoding = obj.get_value("Encoding");
    match encoding {
        None => {
            if font.basename == "Symbol" {
                if font.is_ttot() {
                    font.base_encoding = Some(FontEncoding::MsSymbol);
                } else {
                    font.base_encoding = Some(FontEncoding::AdobeSymbol);
                }
            } else {
                font.base_encoding = Some(FontEncoding::WinAnsi);
            }
        }
        Some(PDFObject::Name(name)) => {
            if font.is_style_symbolic() && font.basename == "Symbol" && !font.is_ttot() {
                font.base_encoding = Some(FontEncoding::AdobeSymbol);
            } else {
                font.base_encoding = get_predefined_encoding(name.name());
            }
        }
        Some(PDFObject::Dictionary(dict)) => {
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
    Ok(())
}

fn load_glyph_map(font: &mut TrueTypeFont, font_dict: &PDFObject) -> PDFResult<()> {
    let base_encoding = font.determain_encoding();
    if (base_encoding == Some(FontEncoding::WinAnsi)
        || base_encoding == Some(FontEncoding::MacRoman))
        && font.diffs.is_empty()
        && !font.is_style_symbolic()
        && font.ft_font.has_glyph_names()
        && font.ft_font.num_charmaps() == 0
    {
        if let Some(startchar) = font_dict.get_value("FirstChar") {
            font.set_glyph_map_from_start(startchar.as_u32()?)
        }
        return Ok(());
    }

    let charmap_type = font.determain_charmap_type()?;
    for charcode in 0..=255 {
        let name: Option<String> = match &base_encoding {
            Some(encoding) => font.charname(charcode, encoding),
            None => None,
        };

        if name.is_none() {
            if let Some(charindex) = font.ft_font.char_index(charcode as usize) {
                font.glphy_index[charcode as usize] = charindex;
                continue;
            }
        }
        // todo
    }

    unimplemented!()
}
pub fn load_to_unicode(obj: &PDFObject) -> PDFResult<ToUnicodeMap> {
    if let Some(tu) = obj.get_value("ToUnicode") {}

    unimplemented!()
}

pub fn create_truetype_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<TrueTypeFont> {
    let mut font = TrueTypeFont::default();
    if let Some(name_res) = obj.get_value_as_string("BaseFont") {
        let name = name_res?;
        font.basename = name.to_string();
    }
    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.get_object_without_indriect(descriptor)?;
        load_font_descriptor(&desc, &mut font)?;
        let ff = desc.get_value("FontFile");
        let ff2 = desc.get_value("FontFile2");
        let ff3 = desc.get_value("FontFile3");
        let fontfile = ff.or(ff2).or(ff3);
        if let Some(program) = fontfile {
            let ft_font = FTFont::try_new(program.bytes()?)?;
            font.ft_font = ft_font;
        }
    }
    load_width(&mut font, obj)?;
    if !font.is_style_symbolic() {
        font.base_encoding = Some(FontEncoding::Standard);
    }
    if !font.ft_font.is_loaded() {
        // TODO load builtin font
    }
    load_encoding(obj, &mut font)?;

    // note must after load descriptor
    unimplemented!()
}
