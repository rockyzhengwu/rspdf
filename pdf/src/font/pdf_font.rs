use crate::error::{PdfError, Result};
use crate::font::simple_font::SimpleFont;
use crate::font::type0::Type0;
use crate::font::CharCode;
use crate::object::dictionary::PdfDict;
use crate::xref::Xref;

use super::{GlyphDesc, WritingMode};

#[derive(Debug, Clone)]
pub enum Font {
    Simple(SimpleFont),
    Type0(Type0),
}

impl Font {
    pub fn try_new(dict: PdfDict, xref: &Xref) -> Result<Self> {
        let subtype = dict
            .get("Subtype")
            .ok_or(PdfError::Font("Subtye is Null".to_string()))?;
        match subtype.as_name()?.name() {
            "Type0" => {
                let type0 = Type0::try_new(dict, xref)?;
                return Ok(Font::Type0(type0));
            }
            "Type1" => {
                let typ1 = SimpleFont::try_new(dict, xref)?;
                return Ok(Font::Simple(typ1));
            }
            "TrueType" => {
                let truetype = SimpleFont::try_new(dict, xref)?;
                return Ok(Font::Simple(truetype));
            }
            "Type3" => {}
            _ => {}
        }
        unimplemented!()
    }

    pub fn text_widths(&self, chars: &[CharCode]) -> Result<f32> {
        match self {
            Font::Simple(t) => t.text_widths(chars),
            Font::Type0(t) => t.text_widths(chars),
        }
    }

    pub fn unicode(&self, char: &CharCode) -> Result<String> {
        match self {
            Font::Simple(t) => t.unicode(char),
            Font::Type0(t) => t.unicode(char),
        }
    }

    pub fn writting_mode(&self) -> WritingMode {
        match self {
            Font::Simple(_) => WritingMode::Horizontal,
            Font::Type0(tf) => tf.writting_mode(),
        }
    }

    pub fn chars(&self, codes: &[u8]) -> Result<Vec<CharCode>> {
        match self {
            Font::Simple(s) => s.chars(codes),
            Font::Type0(ft) => ft.chars(codes),
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Font::Simple(f) => f.base_font(),
            Font::Type0(t) => t.base_font(),
        }
    }

    pub fn get_glyph(&self, char: &CharCode) -> Option<GlyphDesc> {
        match self {
            Font::Simple(s) => s.get_glyph(char),
            Font::Type0(t0) => t0.get_glyph(char),
        }
    }

    pub fn fontfile(&self) -> Option<&[u8]> {
        match self {
            Font::Simple(s) => s.fontfile(),
            Font::Type0(t) => t.fontfile(),
        }
    }
}
