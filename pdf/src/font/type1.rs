use crate::errors::PDFResult;
use crate::font::simple_font::SimpleFont;

fn use_charmap(font: &mut SimpleFont) -> PDFResult<()> {
    let ft_font = font.ft_font();
    if ft_font.use_charmap_platform(7) {
        return Ok(());
    }
    ft_font.use_charmaps_first();
    Ok(())
}

pub fn load_typ1_glyph(font: &mut SimpleFont) -> PDFResult<()> {
    use_charmap(font)?;
    for charcode in 0..255 {
        if let Some(na) = font.charname(charcode) {
            if let Some(glyph) = font.ft_font().find_glyph_by_name(&na) {
                font.set_glyph(charcode, glyph);
            }
        }
    }
    Ok(())
}
