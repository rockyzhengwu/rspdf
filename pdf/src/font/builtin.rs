#![allow(dead_code)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::Path;

use crate::errors::PDFResult;

lazy_static! {
    static ref BUILTINF_FONTS: HashMap<&'static str, &'static str> = {
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

fn load_system_font(name: &str) -> PDFResult<()> {
    let dirs = vec![
        "/usr/share/ghostscript/fonts",
        "/usr/local/share/ghostscript/fonts",
        "/usr/share/fonts/default/Type1",
        "/usr/share/fonts/default/ghostscript",
        "/usr/share/fonts/type1/gsfonts",
    ];
    for d in dirs {
        let path = Path::new(d).join("n021003l.pfb");
        if path.exists() {
            println!("{:?}", path);
        }
    }

    Ok(())
}

pub fn load_builitin_font(name: &str) -> PDFResult<()> {
    if let Some(normal_name) = BUILTINF_FONTS.get(name) {
        // TODO
    } else {
        unimplemented!()
    }

    Ok(())
}
