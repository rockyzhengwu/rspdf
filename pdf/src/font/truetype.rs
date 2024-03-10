use crate::errors::PDFResult;
use crate::font::simple_font::{CharmapType, SimpleFont};
use crate::object::PDFObject;

fn use_charmap(font: &mut SimpleFont) -> PDFResult<CharmapType> {
    let ft_font = font.ft_font();
    if font.is_symbolic() && ft_font.use_charmaps_ms_symbol() {
        return Ok(CharmapType::MsSymbol);
    }
    if ft_font.use_charmaps_ms_unicode() {
        return Ok(CharmapType::MsUnicode);
    }
    if ft_font.use_charmaps_mac_rom() {
        return Ok(CharmapType::MacRoman);
    }
    ft_font.use_charmaps_first();
    Ok(CharmapType::Other)
}

pub fn load_truetype_glyph_map(font: &mut SimpleFont, font_dict: &PDFObject) -> PDFResult<()> {
    if font.is_macrom_or_winasni()
        && !font.has_diffs()
        && !font.is_symbolic()
        && font.has_glyph_names()
        && font.num_charmaps() == 0
    {
        if let Some(startchar) = font_dict.get_value("FirstChar") {
            font.set_glyph_map_from_start(startchar.as_u32()?)
        }
        return Ok(());
    }

    let charmap_type = use_charmap(font)?;
    for charcode in 0..=255 {
        let name = font.charname(charcode);

        if name.is_none() {
            if let Some(charindex) = font.ft_font().char_index(charcode as usize) {
                font.set_glyph(charcode, charindex);
            }
            continue;
        }
        let name = name.unwrap();
        match charmap_type {
            CharmapType::MsSymbol => {
                if let Some(glyph) = font.ft_font().find_glyph_by_name(&name) {
                    font.set_glyph(charcode, glyph);
                }
            }
            CharmapType::MacRoman => {
                if let Some(encoding) = font.base_encoding() {
                    if let Some(mre_code) = encoding.charcode_from_unicode(&(charcode as u16)) {
                        if let Some(glyph) =
                            font.ft_font().find_glyph_by_charindex(mre_code as usize)
                        {
                            font.set_glyph(charcode, glyph);
                        }
                    }
                }
            }
            CharmapType::MsUnicode => {
                if let Some(glyph) = font.ft_font().find_glyph_by_unicode_name(&name) {
                    font.set_glyph(charcode, glyph);
                }
            }
            CharmapType::Other => {}
        }
    }

    Ok(())
}
