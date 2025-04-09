pub mod afm;
pub mod cmap;
pub mod fonts;

pub fn get_builtin_font_data(name: &str) -> Option<&[u8]> {
    match name {
        "Courier" => Some(fonts::n022003l::DATA),
        "Courier-Bold" => Some(fonts::n022004l::DATA),
        "Courier-BoldOblique" => Some(fonts::n022024l::DATA),
        "Courier-Oblique" => Some(fonts::n022023l::DATA),
        "Helvetica" => Some(fonts::n019003l::DATA),
        "Helvetica-Bold" => Some(fonts::n019004l::DATA),
        "Helvetica-BoldOblique" => Some(fonts::n019024l::DATA),
        "Helvetica-Oblique" => Some(fonts::n019023l::DATA),
        "Symbol" => Some(fonts::s050000l::DATA),
        "Times-Bold" => Some(fonts::n021004l::DATA),
        "Times-BoldItalic" => Some(fonts::n021024l::DATA),
        "Times-Italic" => Some(fonts::n021023l::DATA),
        "Times-Roman" => Some(fonts::n021003l::DATA),
        "ZapfDingbats" => Some(fonts::d050000l::DATA),
        _ => None,
    }
}

pub fn get_builtin_font_matrices(name: &str) -> Option<&[u8]> {
    match name {
        "Courier" => Some(afm::courier::DATA),
        "Courier-Bold" => Some(afm::courier_bold::DATA),
        "Courier-BoldOblique" => Some(afm::courier_boldoblique::DATA),
        "Courier-Oblique" => Some(afm::courier_oblique::DATA),
        "Helvetica" => Some(afm::helvetica::DATA),
        "Helvetica-Bold" => Some(afm::helvetica_bold::DATA),
        "Helvetica-BoldOblique" => Some(afm::helvetica_boldoblique::DATA),
        "Helvetica-Oblique" => Some(afm::helvetica_oblique::DATA),
        "Symbol" => Some(afm::symbol::DATA),
        "Times-Bold" => Some(afm::times_bold::DATA),
        "Times-BoldItalic" => Some(afm::times_bolditalic::DATA),
        "Times-Italic" => Some(afm::times_italic::DATA),
        "Times-Roman" => Some(afm::times_roman::DATA),
        "ZapfDingbats" => Some(afm::zapfdingbats::DATA),
        _ => None,
    }
}
