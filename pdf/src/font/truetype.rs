use crate::errors::PDFResult;
use crate::font::encoding::FontEncoding;
use crate::font::simple_font::{CharmapType, SimpleFont};
use crate::object::PDFObject;



pub fn load_truetype_glyph_map(font: &mut SimpleFont, font_dict: &PDFObject) -> PDFResult<()> {
    let base_encoding = font.determain_encoding();
    let ft_font = font.ft_font();
    if (base_encoding == Some(FontEncoding::WinAnsi)
        || base_encoding == Some(FontEncoding::MacRoman))
        && !font.has_diffs()
        && !font.is_symbolic()
        && ft_font.has_glyph_names()
        && ft_font.num_charmaps() == 0
    {
        if let Some(startchar) = font_dict.get_value("FirstChar") {
            font.set_glyph_map_from_start(startchar.as_u32()?)
        }
        return Ok(());
    }

    let charmap_type = font.determain_charmap_type()?;
    for charcode in 0..=255 {
        let name: Option<String> = match &base_encoding {
            Some(encoding) => font.charname(charcode, encoding),
            None => None,
        };

        if name.is_none() {
            if let Some(charindex) = ft_font.char_index(charcode as usize) {
                font.set_glyph(charcode, charindex);
            }
            continue;
        }
        let name = name.unwrap();
        match charmap_type {
            CharmapType::MsSymbol => {
                if let Some(glyph) = ft_font.find_glyph_by_name(&name) {
                    font.set_glyph(charcode, glyph);
                }
            }
            CharmapType::MacRoman => {
                if let Some(encoding) = base_encoding {
                    if let Some(mre_code) = encoding.charcode_from_unicode(&(charcode as u16)) {
                        if let Some(glyph) = ft_font.find_glyph_by_charindex(mre_code as usize) {
                            font.set_glyph(charcode, glyph);
                        }
                    }
                }
            }
            CharmapType::MsUnicode => {
                if let Some(glyph) = ft_font.find_glyph_by_unicode_name(&name) {
                    font.set_glyph(charcode, glyph);
                }
            }
            CharmapType::Other => {}
        }
    }

    Ok(())
}
