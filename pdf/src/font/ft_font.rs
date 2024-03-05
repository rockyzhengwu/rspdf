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
}
