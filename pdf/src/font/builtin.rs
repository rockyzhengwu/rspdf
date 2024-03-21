use std::collections::HashMap;
use std::path::{Path, PathBuf};

use freetype::face::Face;
use freetype::Library;
use lazy_static::lazy_static;

use crate::errors::{PDFError, PDFResult};

lazy_static! {
    static ref BUILTINF_FONTS_NAME: HashMap<&'static str, &'static str> = {
        let mut name_map = HashMap::new();
        name_map.insert("Arial", "Helvetica");
        name_map.insert("Arial,Bold", "Helvetica-Bold");
        name_map.insert("Arial,BoldItalic", "Helvetica-BoldOblique");
        name_map.insert("Arial,Italic", "Helvetica-Oblique");
        name_map.insert("Arial-Bold", "Helvetica-Bold");
        name_map.insert("Arial-BoldItalic", "Helvetica-BoldOblique");
        name_map.insert("Arial-BoldItalicMT", "Helvetica-BoldOblique");
        name_map.insert("Arial-BoldMT", "Helvetica-Bold");
        name_map.insert("Arial-Italic", "Helvetica-Oblique");
        name_map.insert("Arial-ItalicMT", "Helvetica-Oblique");
        name_map.insert("ArialMT", "Helvetica");
        name_map.insert("Courier", "Courier");
        name_map.insert("Courier,Bold", "Courier-Bold");
        name_map.insert("Courier,BoldItalic", "Courier-BoldOblique");
        name_map.insert("Courier,Italic", "Courier-Oblique");
        name_map.insert("Courier-Bold", "Courier-Bold");
        name_map.insert("Courier-BoldOblique", "Courier-BoldOblique");
        name_map.insert("Courier-Oblique", "Courier-Oblique");
        name_map.insert("CourierNew", "Courier");
        name_map.insert("CourierNew,Bold", "Courier-Bold");
        name_map.insert("CourierNew,BoldItalic", "Courier-BoldOblique");
        name_map.insert("CourierNew,Italic", "Courier-Oblique");
        name_map.insert("CourierNew-Bold", "Courier-Bold");
        name_map.insert("CourierNew-BoldItalic", "Courier-BoldOblique");
        name_map.insert("CourierNew-Italic", "Courier-Oblique");
        name_map.insert("CourierNewPS-BoldItalicMT", "Courier-BoldOblique");
        name_map.insert("CourierNewPS-BoldMT", "Courier-Bold");
        name_map.insert("CourierNewPS-ItalicMT", "Courier-Oblique");
        name_map.insert("CourierNewPSMT", "Courier");
        name_map.insert("Helvetica", "Helvetica");
        name_map.insert("Helvetica,Bold", "Helvetica-Bold");
        name_map.insert("Helvetica,BoldItalic", "Helvetica-BoldOblique");
        name_map.insert("Helvetica,Italic", "Helvetica-Oblique");
        name_map.insert("Helvetica-Bold", "Helvetica-Bold");
        name_map.insert("Helvetica-BoldItalic", "Helvetica-BoldOblique");
        name_map.insert("Helvetica-BoldOblique", "Helvetica-BoldOblique");
        name_map.insert("Helvetica-Italic", "Helvetica-Oblique");
        name_map.insert("Helvetica-Oblique", "Helvetica-Oblique");
        name_map.insert("Symbol", "Symbol");
        name_map.insert("Symbol,Bold", "Symbol");
        name_map.insert("Symbol,BoldItalic", "Symbol");
        name_map.insert("Symbol,Italic", "Symbol");
        name_map.insert("Times-Bold", "Times-Bold");
        name_map.insert("Times-BoldItalic", "Times-BoldItalic");
        name_map.insert("Times-Italic", "Times-Italic");
        name_map.insert("Times-Roman", "Times-Roman");
        name_map.insert("TimesNewRoman", "Times-Roman");
        name_map.insert("TimesNewRoman,Bold", "Times-Bold");
        name_map.insert("TimesNewRoman,BoldItalic", "Times-BoldItalic");
        name_map.insert("TimesNewRoman,Italic", "Times-Italic");
        name_map.insert("TimesNewRoman-Bold", "Times-Bold");
        name_map.insert("TimesNewRoman-BoldItalic", "Times-BoldItalic");
        name_map.insert("TimesNewRoman-Italic", "Times-Italic");
        name_map.insert("TimesNewRomanPS", "Times-Roman");
        name_map.insert("TimesNewRomanPS-Bold", "Times-Bold");
        name_map.insert("TimesNewRomanPS-BoldItalic", "Times-BoldItalic");
        name_map.insert("TimesNewRomanPS-BoldItalicMT", "Times-BoldItalic");
        name_map.insert("TimesNewRomanPS-BoldMT", "Times-Bold");
        name_map.insert("TimesNewRomanPS-Italic", "Times-Italic");
        name_map.insert("TimesNewRomanPS-ItalicMT", "Times-Italic");
        name_map.insert("TimesNewRomanPSMT", "Times-Roman");
        name_map.insert("TimesNewRomanPSMT,Bold", "Times-Bold");
        name_map.insert("TimesNewRomanPSMT,BoldItalic", "Times-BoldItalic");
        name_map.insert("TimesNewRomanPSMT,Italic", "Times-Italic");
        name_map.insert("ZapfDingbats", "ZapfDingbats");
        name_map
    };
}
lazy_static! {
    static ref SYS_FONTS_FILE: HashMap<&'static str, &'static str> = {
        let mut name_map = HashMap::new();
        name_map.insert("Courier", "n022003l.pfb");
        name_map.insert("Courier-Bold", "n022004l.pfb");
        name_map.insert("Courier-BoldOblique", "n022024l.pfb");
        name_map.insert("Courier-Oblique", "n022023l.pfb");
        name_map.insert("Helvetica", "n019003l.pfb");
        name_map.insert("Helvetica-Bold", "n019004l.pfb");
        name_map.insert("Helvetica-BoldOblique", "n019024l.pfb");
        name_map.insert("Helvetica-Oblique", "n019023l.pfb");
        name_map.insert("Symbol", "s050000l.pfb");
        name_map.insert("Times-Bold", "n021004l.pfb");
        name_map.insert("Times-BoldItalic", "n021024l.pfb");
        name_map.insert("Times-Italic", "n021023l.pfb");
        name_map.insert("Times-Roman", "n021003l.pfb");
        name_map.insert("ZapfDingbats", "d050000l.pfb");
        name_map
    };
}

fn load_face(path: PathBuf) -> PDFResult<Face> {
    let lib = Library::init().unwrap();
    match lib.new_face(path, 0) {
        Ok(face) => Ok(face),
        Err(e) => Err(PDFError::FontFreeType(format!("Load face error{:?}", e))),
    }
}

fn load_system_font(name: &str) -> PDFResult<Option<Face>> {
    let dirs = vec![
        "/usr/share/ghostscript/fonts",
        "/usr/local/share/ghostscript/fonts",
        "/usr/share/fonts/default/Type1",
        "/usr/share/fonts/default/ghostscript",
        "/usr/share/fonts/type1/gsfonts",
    ];
    if let Some(fname) = SYS_FONTS_FILE.get(name) {
        for d in dirs {
            let path = Path::new(d).join(fname);
            if path.exists() {
                let face = load_face(path)?;
                return Ok(Some(face));
            }
        }
    } else {
        // TODO support other system, just linux now
        panic!("Built in fonts not  found:{:?}", name);
    }
    panic!("built in fonts not found:{:?}", name);
}

pub fn load_memory_face(bytes: &[u8]) -> PDFResult<Face> {
    let lib = Library::init().unwrap();
    match lib.new_memory_face(bytes.to_vec(), 0) {
        Ok(face) => Ok(face),
        Err(e) => Err(PDFError::FontFreeType(format!("Load face error{:?}", e))),
    }
}

pub fn load_base14_font(name: &str) -> PDFResult<Option<Face>> {
    if let Some(font_data) = font_data::get_builtin_font_data(name) {
        let face = load_memory_face(font_data)?;
        return Ok(Some(face));
    }
    Ok(None)
}

pub fn load_builtin_font(name: &str) -> PDFResult<Option<Face>> {
    match BUILTINF_FONTS_NAME.get(name) {
        Some(n) => load_base14_font(n),
        // TODO find substitute font instead of defalut
        None => load_base14_font("Helvetica"),
    }
}
