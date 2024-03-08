use freetype::charmap::CharMap;
// freetype font helper
use freetype::face::Face;
use freetype::ffi::FT_FACE_FLAG_SFNT;
use freetype::library::Library;

use crate::errors::{PDFError, PDFResult};

#[derive(Default)]
pub struct FTFont {
    face: Option<Face>,
}

impl FTFont {
    pub fn try_new(buffer: Vec<u8>) -> PDFResult<Self> {
        // TODO library init globaly
        let lib = Library::init().unwrap();
        match lib.new_memory_face(buffer, 0) {
            Ok(face) => Ok(FTFont { face: Some(face) }),
            Err(e) => Err(PDFError::FontFreeType(format!("Load face error{:?}", e))),
        }
    }
    pub fn is_loaded(&self) -> bool {
        self.face.is_some()
    }

    pub fn is_ttot(&self) -> bool {
        if let Some(face) = &self.face {
            return (face.raw().face_flags & FT_FACE_FLAG_SFNT) == 0;
        }
        false
    }

    pub fn num_charmaps(&self) -> i32 {
        match self.face {
            Some(ref f) => f.num_charmaps(),
            None => 0,
        }
    }

    pub fn charmap(&self, index: i32) -> Option<CharMap> {
        self.face.as_ref().map(|f| f.get_charmap(index as isize))
    }

    pub fn has_glyph_names(&self) -> bool {
        match self.face {
            Some(ref f) => f.has_glyph_names(),
            None => false,
        }
    }

    fn use_tt_charmap(&self, platform_id: u16, encoding_id: u16) -> bool {
        if let Some(ref face) = self.face {
            let num_charmaps = self.num_charmaps();
            for i in 0..num_charmaps {
                let charmap = face.get_charmap(i as isize);
                if charmap.platform_id() == platform_id && charmap.encoding_id() == encoding_id {
                    // TODO handle this error
                    face.set_charmap(&charmap).unwrap();
                    return true;
                }
            }
        }
        false
    }
    pub fn use_charmaps_ms_unicode(&self) -> bool {
        self.use_tt_charmap(3, 1)
    }
    pub fn use_charmaps_ms_symbol(&self) -> bool {
        self.use_tt_charmap(3, 0)
    }
    pub fn use_charmaps_mac_rom(&self) -> bool {
        self.use_tt_charmap(1, 0)
    }

    pub fn char_index(&self, charcode: usize) -> Option<u32> {
        match &self.face {
            Some(face) => face.get_char_index(charcode),
            None => None,
        }
    }
}
