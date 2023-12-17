use std::collections::HashMap;
use std::fmt::{self, Debug};

use freetype::{Bitmap, Face};

use crate::font::cid_font::CIDFont;
use crate::font::cmap::CMap;
use crate::object::{PDFObject, PDFString};

const GLYPH_SPACE: f64 = 1000.0;

#[derive(Clone, Default)]
pub struct SimpleFont {
    #[allow(dead_code)]
    name: String,
    obj: PDFObject,
    tounicode: CMap,
    widths: HashMap<u32, u32>,
    face: Option<Face>,
    cid: Option<CIDFont>,
}

impl SimpleFont {
    pub fn new(
        name: &str,
        obj: PDFObject,
        tounicode: CMap,
        widths: HashMap<u32, u32>,
        face: Option<Face>,
        cid: Option<CIDFont>,
    ) -> Self {
        SimpleFont {
            name: name.to_string(),
            obj,
            tounicode,
            widths,
            face,
            cid,
        }
    }

    pub fn get_char_with(&self, code: &u32) -> u32 {
        self.widths.get(code).unwrap_or(&0_u32).to_owned()
    }

    pub fn get_content_width(
        &self,
        content: &PDFString,
        font_size: f64,
        char_spacing: f64,
        hscaling: f64,
    ) -> f64 {
        let mut width = 0.0;
        // TODO, PDFString to String need to condider font
        match content {
            PDFString::Literial(s) => {
                for ch in s {
                    let code = ch.to_owned() as u32;
                    let w = self.widths.get(&code).unwrap_or(&0).to_owned() as f64 / GLYPH_SPACE;
                    width += w.to_owned() * font_size + char_spacing + hscaling / GLYPH_SPACE;
                }
            }
            _ => {
                width = 0.0;
            }
        }
        width
    }

    pub fn decode_to_glyph(&self, code: u32, sx: u32, sy: u32) -> Bitmap {
        match self.face {
            Some(ref f) => {
                f.set_pixel_sizes(sx, sy).unwrap();
                if let Some(ref cid) = self.cid {
                    let gid = cid.map_code_gid(code);
                    f.load_glyph(gid, freetype::face::LoadFlag::RENDER).unwrap();
                    let glyph = f.glyph();
                    glyph.bitmap()
                } else {
                    f.load_char(code as usize, freetype::face::LoadFlag::RENDER)
                        .unwrap();
                    let glyph = f.glyph();
                    glyph.bitmap()
                }
            }
            None => {
                panic!("face is None in font");
            }
        }
    }

    pub fn get_unicode(&self, content: &PDFString) -> String {
        // TODO fix this
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
