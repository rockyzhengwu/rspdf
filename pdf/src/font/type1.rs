use crate::errors::PDFResult;
use crate::font::simple_font::SimpleFont;

pub fn load_typ1_glyph(font: &mut SimpleFont) -> PDFResult<()> {
    let base_encoding = font.determain_encoding();
    let ft_font = font.ft_font();
    for charcode in 0..255 {
        let name: Option<String> = match &base_encoding {
            Some(encoding) => font.charname(charcode, encoding),
            None => None,
        };
        if let Some(na) = name {
            if let Some(glyph) = ft_font.find_glyph_by_name(&na) {
                font.set_glyph(charcode, glyph);
            }
        }
    }
    Ok(())
}
