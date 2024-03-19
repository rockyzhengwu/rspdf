use freetype::charmap::CharMap;
use freetype::ffi::FT_FACE_FLAG_SFNT;
use freetype::GlyphSlot;
use freetype::{face::LoadFlag, Face, Library};

use crate::errors::{PDFError, PDFResult};
use crate::font::builtin::load_builitin_font;
use crate::font::glyph_name::name_to_unicode;

#[derive(Default, Debug, Clone)]
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
    pub fn try_new_builtin(name: &str) -> PDFResult<Self> {
        let face = load_builitin_font(name)?;
        Ok(FTFont { face })
    }

    pub fn get_glyph(&self, gid: u32, scale: u32) -> Option<GlyphSlot> {
        match &self.face {
            Some(f) => {
                f.set_pixel_sizes(scale.to_owned(), scale.to_owned())
                    .unwrap();
                f.load_glyph(gid, LoadFlag::RENDER).unwrap();
                let glyph = f.glyph();
                Some(glyph.to_owned())
            }
            None => None,
        }
    }

    pub fn ft_face(&self) -> Option<&Face> {
        self.face.as_ref()
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

    pub fn use_charmaps_first(&self) -> bool {
        if let Some(charmap) = self.charmap(0) {
            match &self.face {
                Some(face) => {
                    face.set_charmap(&charmap).unwrap();
                    return true;
                }
                None => return false,
            }
        }
        false
    }
    pub fn use_charmap_platform(&self, platform: u16) -> bool {
        if let Some(face) = &self.face {
            let num_chamaps = self.num_charmaps();
            for i in 0..num_chamaps {
                let charmap = face.get_charmap(i as isize);
                if charmap.platform_id() == platform {
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

    pub fn find_glyph_by_unicode_name(&self, name: &str) -> Option<u32> {
        if let Some(unicode) = name_to_unicode(name) {
            return self.find_glyph_by_charindex(unicode as usize);
        }
        None
    }
    pub fn find_glyph_by_name(&self, name: &str) -> Option<u32> {
        if let Some(face) = &self.face {
            return face.get_name_index(name);
        }
        None
    }
    pub fn find_glyph_by_charindex(&self, charindex: usize) -> Option<u32> {
        if let Some(face) = &self.face {
            return face.get_char_index(charindex);
        }
        None
    }
}
