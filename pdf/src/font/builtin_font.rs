use std::collections::HashMap;

use font_data::{get_builtin_font_data, get_builtin_font_matrices};
use lazy_static::lazy_static;

use super::CharCode;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BuiltEncoding {
    Standard,
    MacRoman,
    WinAnsi,
    PdfDoc,
    MacExpert,
    AdobeSymbol,
    ZapfDingbats,
    MsSymbol,
}

pub fn get_char_width(char: CharCode) -> Option<f32> {
    unimplemented!()
}

pub fn load_builtin_font_data(name: &str) -> Option<&[u8]> {
    match BASE_14_FONTNAME.get(name) {
        Some(fname) => get_builtin_font_data(fname),
        None => get_builtin_font_data("Helvetica"),
    }
}

pub fn load_builtin_metrics(name: &str) -> Option<&[u8]> {
    get_builtin_font_matrices(name)
}

pub fn find_builtin_font(name: &str) -> Option<&str> {
    match BASE_14_FONTNAME.get(name) {
        Some(s) => Some(s.to_owned()),
        None => None,
    }
}

lazy_static! {
    static ref BASE_14_FONTNAME: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();

        map.insert("Arial", "Helvetica");
        map.insert("Arial,Bold", "Helvetica-Bold");
        map.insert("Arial,BoldItalic", "Helvetica-BoldOblique");
        map.insert("Arial,Italic", "Helvetica-Oblique");
        map.insert("Arial-Bold", "Helvetica-Bold");
        map.insert("Arial-BoldItalic", "Helvetica-BoldOblique");
        map.insert("Arial-BoldItalicMT", "Helvetica-BoldOblique");
        map.insert("Arial-BoldMT", "Helvetica-Bold");
        map.insert("Arial-Italic", "Helvetica-Oblique");
        map.insert("ArialMT", "Helvetica");
        map.insert("Arial-ItalicMT", "Helvetica-Oblique");
        map.insert("Courier", "Courier");
        map.insert("Courier,Bold", "Courier-Bold");
        map.insert("Courier,BoldItalic", "Courier-BoldOblique");
        map.insert("Courier,Italic", "Courier-Oblique");
        map.insert("Courier-Bold", "Courier-Bold");
        map.insert("Courier-BoldOblique", "Courier-BoldOblique");
        map.insert("CourierNew", "Courier");
        map.insert("Courier-Oblique", "Courier-Oblique");
        map.insert("CourierNew,Bold", "Courier-Bold");
        map.insert("CourierNew,BoldItalic", "Courier-BoldOblique");
        map.insert("CourierNew,Italic", "Courier-Oblique");
        map.insert("CourierNew-Bold", "Courier-Bold");
        map.insert("CourierNew-BoldItalic", "Courier-BoldOblique");
        map.insert("CourierNew-Italic", "Courier-Oblique");
        map.insert("CourierNewPS-BoldItalicMT", "Courier-BoldOblique");
        map.insert("CourierNewPS-BoldMT", "Courier-Bold");
        map.insert("CourierNewPS-ItalicMT", "Courier-Oblique");
        map.insert("CourierNewPSMT", "Courier");
        map.insert("Helvetica", "Helvetica");
        map.insert("Helvetica,Bold", "Helvetica-Bold");
        map.insert("Helvetica,BoldItalic", "Helvetica-BoldOblique");
        map.insert("Helvetica,Italic", "Helvetica-Oblique");
        map.insert("Helvetica-BoldItalic", "Helvetica-BoldOblique");
        map.insert("Helvetica-Bold", "Helvetica-Bold");
        map.insert("Helvetica-BoldOblique", "Helvetica-BoldOblique");
        map.insert("Helvetica-Italic", "Helvetica-Oblique");
        map.insert("Helvetica-Oblique", "Helvetica-Oblique");
        map.insert("Symbol,Bold", "Symbol");
        map.insert("Symbol", "Symbol");
        map.insert("Symbol,BoldItalic", "Symbol");
        map.insert("Symbol,Italic", "Symbol");
        map.insert("Times-Bold", "Times-Bold");
        map.insert("Times-BoldItalic", "Times-BoldItalic");
        map.insert("Times-Roman", "Times-Roman");
        map.insert("Times-Italic", "Times-Italic");
        map.insert("TimesNewRoman,Bold", "Times-Bold");
        map.insert("TimesNewRoman", "Times-Roman");
        map.insert("TimesNewRoman,BoldItalic", "Times-BoldItalic");
        map.insert("TimesNewRoman,Italic", "Times-Italic");
        map.insert("TimesNewRoman-Bold", "Times-Bold");
        map.insert("TimesNewRoman-BoldItalic", "Times-BoldItalic");
        map.insert("TimesNewRoman-Italic", "Times-Italic");
        map.insert("TimesNewRomanPS", "Times-Roman");
        map.insert("TimesNewRomanPS-Bold", "Times-Bold");
        map.insert("TimesNewRomanPS-BoldItalic", "Times-BoldItalic");
        map.insert("TimesNewRomanPS-BoldItalicMT", "Times-BoldItalic");
        map.insert("TimesNewRomanPS-BoldMT", "Times-Bold");
        map.insert("TimesNewRomanPS-Italic", "Times-Italic");
        map.insert("TimesNewRomanPS-ItalicMT", "Times-Italic");
        map.insert("TimesNewRomanPSMT", "Times-Roman");
        map.insert("TimesNewRomanPSMT,Bold", "Times-Bold");
        map.insert("TimesNewRomanPSMT,Italic", "Times-Italic");
        map.insert("TimesNewRomanPSMT,BoldItalic", "Times-BoldItalic");
        map.insert("ZapfDingbats", "ZapfDingbats");
        map
    };
}
