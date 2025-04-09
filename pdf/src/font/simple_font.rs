use std::collections::HashMap;

use crate::error::{PdfError, Result};
use crate::font::afm::Afm;
use crate::font::cmap::Cmap;
use crate::font::descriptor::Descriptor;
use crate::font::encoding::Encoding;
use crate::font::CharCode;
use crate::object::dictionary::PdfDict;
use crate::object::PdfObject;
use crate::xref::Xref;

use super::afm::parse_afm;
use super::builtin_font::{find_builtin_font, load_builtin_font_data, load_builtin_metrics};
use super::encoding::stand_mac_roman_name_to_unicode;
use super::font_program::{load_freetype_face, use_charmap, CharmapType};
use super::glyph_name::adobe_glyph_list_to_unicode;
use super::GlyphDesc;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SimpleSubType {
    TrueType,
    #[default]
    Type1,
    MMType1,
}

#[derive(Debug, Default, Clone)]
pub struct SimpleFont {
    dict: PdfDict,
    base_font: String,
    to_unicode: Option<Cmap>,
    descriptor: Descriptor,
    first_char: Option<u32>,
    last_char: Option<u32>,
    widths: Option<[f32; 256]>,
    encoding: Option<Encoding>,
    is_embed: bool,
    subtype: SimpleSubType,
    code_to_gid: HashMap<u8, u32>,
    afm: Option<Afm>,
}

impl SimpleFont {
    pub fn base_font(&self) -> &str {
        self.base_font.as_str()
    }

    pub fn try_new(dict: PdfDict, xref: &Xref) -> Result<Self> {
        let subtype = match dict.get("Subtype") {
            Some(st) => {
                let tn = st.as_name()?.name();
                match tn {
                    "TrueType" => SimpleSubType::TrueType,
                    "Type1" => SimpleSubType::Type1,
                    "MMType1" => SimpleSubType::MMType1,
                    _ => {
                        return Err(PdfError::Font(
                            "Simple font subtype  is invalid".to_string(),
                        ))
                    }
                }
            }
            None => {
                return Err(PdfError::Font("Simple font subtype is None".to_string()));
            }
        };
        let mut font = SimpleFont {
            dict,
            subtype,
            is_embed: false,
            ..Default::default()
        };

        if let Some(bf) = font.dict.get("BaseFont") {
            let name = bf.as_name()?;
            font.base_font = name.name().to_string();
        }

        if let Some(first_char) = font.dict.get("FirstChar") {
            font.first_char = Some(first_char.integer()? as u32);
        }

        if let Some(last_char) = font.dict.get("LastChar") {
            font.last_char = Some(last_char.integer()? as u32);
        }
        font.load_descriptor(xref)?;
        font.load_encoding(xref)?;
        font.load_to_unicode(xref)?;
        font.load_width(xref)?;
        font.init_cid_to_gid()?;

        // TODO code to gid

        Ok(font)
    }
    fn code_to_name(&self, code: u8) -> Option<&str> {
        match &self.encoding {
            Some(enc) => enc.get_glyph_name(code),
            None => None,
        }
    }

    fn init_truetype_cid_to_gid(&mut self) -> Result<()> {
        if let Some(fontfile) = self.fontfile() {
            let face = load_freetype_face(fontfile.to_vec())?;
            let charmap_type = use_charmap(self.descriptor.is_symbolic(), &face)?;

            for charcode in 0..=255 {
                if let Some(gname) = self.code_to_name(charcode) {
                    match charmap_type {
                        CharmapType::MsSymbol => {
                            let idx = charcode as u16 + 0xf00;
                            let gid = face.get_char_index(idx as usize).unwrap_or(0);
                            self.code_to_gid.insert(charcode, gid);
                        }
                        CharmapType::MacRoman => {
                            let unicode = stand_mac_roman_name_to_unicode(charcode).unwrap_or(0);
                            let gid = face.get_char_index(unicode as usize).unwrap_or(0);
                            self.code_to_gid.insert(charcode, gid);
                        }
                        CharmapType::MsUnicode => {
                            let unicode = adobe_glyph_list_to_unicode(gname).unwrap_or(0);
                            let gid = face.get_char_index(unicode as usize).unwrap_or(0);
                            self.code_to_gid.insert(charcode, gid);
                        }
                        CharmapType::Other => {
                            let gid = face.get_name_index(gname).unwrap_or(0);
                            self.code_to_gid.insert(charcode, gid);
                        }
                    }
                } else {
                    match charmap_type {
                        CharmapType::MsSymbol => {
                            let gid = face.get_char_index(0xF000 + charcode as usize).unwrap_or(0);
                            self.code_to_gid.insert(charcode, gid);
                        }
                        _ => {
                            let gid = face.get_char_index(charcode as usize).unwrap_or(0);
                            self.code_to_gid.insert(charcode, gid);
                        }
                    }
                }
            }
        } else {
            println!("fontfile is none");
        }
        Ok(())
    }

    fn init_type1_cid_to_gid(&mut self) -> Result<()> {
        if let Some(fontfile) = self.fontfile() {
            let face = load_freetype_face(fontfile.to_vec())?;
            for charcode in 0..=255 {
                if let Some(enc) = &self.encoding {
                    if let Some(name) = enc.get_glyph_name(charcode) {
                        let gid = face.get_name_index(name).unwrap_or(0);
                        self.code_to_gid.insert(charcode, gid);
                    } else {
                        let gid = face.get_char_index(charcode as usize).unwrap_or(0);
                        self.code_to_gid.insert(charcode, gid);
                    }
                } else {
                    let gid = face.get_char_index(charcode as usize).unwrap_or(0);
                    self.code_to_gid.insert(charcode, gid);
                }
            }
        }
        Ok(())
    }

    fn init_cid_to_gid(&mut self) -> Result<()> {
        match self.subtype {
            SimpleSubType::Type1 | SimpleSubType::MMType1 => self.init_type1_cid_to_gid(),
            SimpleSubType::TrueType => self.init_truetype_cid_to_gid(),
        }
    }

    pub fn chars(&self, codes: &[u8]) -> Result<Vec<CharCode>> {
        let mut res = Vec::new();
        for c in codes {
            let width = self.char_width(c.to_owned())?;
            let ch = CharCode::new(c.to_owned() as u32, 1, width);
            res.push(ch);
        }
        Ok(res)
    }

    fn load_to_unicode(&mut self, xref: &Xref) -> Result<()> {
        if let Some(tu) = self.dict.get("ToUnicode") {
            match tu {
                PdfObject::Indirect(_) => {
                    let tobj = xref.read_object(tu)?.to_stream()?;
                    let cmap = Cmap::try_new(tobj.decode_data(Some(xref))?)?;
                    self.to_unicode = Some(cmap);
                }
                PdfObject::Stream(tus) => {}
                _ => {}
            }
        }
        Ok(())
    }

    fn load_width(&mut self, xref: &Xref) -> Result<()> {
        if let Some(ws) = self.dict.get("Widths") {
            let ws = xref.read_object(ws)?;
            let wsa = ws.as_array()?;
            let mut widths = [0.0; 256];

            for (i, v) in wsa.iter().enumerate() {
                let index = i + self.first_char.unwrap() as usize;
                widths[index] = v.as_number()?.real();
            }
            self.widths = Some(widths);
        }
        if let Some(name) = find_builtin_font(self.base_font()) {
            if let Some(afm_data) = load_builtin_metrics(name) {
                let s = String::from_utf8(afm_data.to_vec())
                    .map_err(|_| PdfError::Font("afm data not utf8".to_string()))?;
                let afm = parse_afm(s)?;
                self.afm = Some(afm);
            }
        }
        Ok(())
    }

    fn load_descriptor(&mut self, xref: &Xref) -> Result<()> {
        if let Some(desc) = self.dict.get("FontDescriptor") {
            match desc {
                PdfObject::Indirect(_) => {
                    let desc_dict = xref.read_object(desc)?.to_dict()?;
                    let desc = Descriptor::try_new(desc_dict, xref)?;
                    if desc.fontfile().is_some() {
                        self.is_embed = true;
                    }
                    self.descriptor = desc;
                }
                _ => {
                    return Err(PdfError::Font(
                        "Simple Fopnt Descriptor must be a Indirect or Dict ".to_string(),
                    ))
                }
            };
        } else {
            self.is_embed = true;
        }
        Ok(())
    }

    fn load_encoding(&mut self, xref: &Xref) -> Result<()> {
        if let Some(enc) = self.dict.get("Encoding") {
            match enc {
                PdfObject::Dict(d) => {
                    let encoding = Encoding::try_new(d)?;
                    self.encoding = Some(encoding);
                }
                PdfObject::Indirect(_) => {
                    let encd = xref.read_object(enc)?.to_dict()?;
                    let encoding = Encoding::try_new(&encd)?;
                    self.encoding = Some(encoding);
                }
                PdfObject::Name(name) => {
                    let encoding = Encoding::new_from_name(name.name())?;
                    self.encoding = Some(encoding);
                }
                _ => {
                    return Err(PdfError::Font(
                        "Simple Font Encoding need Name Dict or Indirect".to_string(),
                    ))
                }
            }
        }
        if self.encoding.is_none() {
            // TODO load encoding from font program, implement type1 font parser
        }

        // TODO set default encoding
        if self.encoding.is_none() {
            if let Some(builtin_name) = find_builtin_font(self.base_font.as_str()) {
                if builtin_name == "Symbol" {
                    let encoding = Encoding::new_from_name("Symbol")?;
                    self.encoding = Some(encoding);
                } else if builtin_name == "ZapfDingbats" {
                    let encoding = Encoding::new_from_name("ZapfDingbats")?;
                    self.encoding = Some(encoding);
                } else {
                    let encoding = Encoding::new_from_name("StandardEncoding")?;
                    self.encoding = Some(encoding);
                }
            }
        }
        Ok(())
    }

    pub fn char_width(&self, code: u8) -> Result<f32> {
        if let Some(widths) = self.widths {
            return Ok(widths[code.to_owned() as usize]);
        }
        if let Some(afm) = self.afm.as_ref() {
            if let Some(enc) = &self.encoding {
                if let Some(name) = enc.get_glyph_name(code) {
                    if let Some(w) = afm.get_char_width(name) {
                        return Ok(w);
                    }
                }
            }
        }
        return Err(PdfError::Font("Font width is erro".to_string()));
    }

    pub fn unicode(&self, ch: &CharCode) -> Result<String> {
        // TODO handle simple font to_unicode is None
        match &self.to_unicode {
            Some(cmap) => {
                let u = cmap.unicode(ch).unwrap();
                Ok(u.to_string())
            }
            None => match &self.encoding {
                Some(enc) => {
                    let u = enc.unicode_from_charcode(&(ch.code() as u8));
                    let c = char::from_u32(u).unwrap();
                    let mut s = String::new();
                    s.push(c);
                    Ok(s)
                }
                None => {
                    //let mut fnf = std::fs::File::create("truetype_without_encoding.ttf").unwrap();
                    Err(PdfError::Font("Simple Font to_unicode is None".to_string()))
                }
            },
        }
    }

    pub fn text_widths(&self, chars: &[CharCode]) -> Result<f32> {
        let mut total_width: f32 = 0.0;
        for c in chars {
            let w = self.char_width(c.code() as u8)?;
            total_width += w;
        }
        Ok(total_width)
    }

    pub fn get_glyph(&self, code: &CharCode) -> Option<GlyphDesc> {
        let c = code.code() as u8;
        if let Some(gid) = self.code_to_gid.get(&c) {
            return Some(GlyphDesc::Gid(gid.to_owned()));
        }
        println!(
            "result is None:{:?},{:?},{:?},{:?}",
            code,
            self.code_to_gid,
            self.subtype,
            self.base_font()
        );
        None

        // encoding 获取 glypn_name ,用 name 去查 glyph
    }

    pub fn fontfile(&self) -> Option<&[u8]> {
        if let Some(o) = self.descriptor.fontfile() {
            Some(o)
        } else {
            load_builtin_font_data(&self.base_font)
        }
    }
}
