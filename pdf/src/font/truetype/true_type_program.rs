use crate::error::Result;
use crate::font::truetype::{reader::TrueTypeReader, sfnt::Sfnt, tables::cmap::TCmap};

#[derive(Debug, Clone)]
pub struct TrueTypeProgram {
    cmaps: Vec<TCmap>,
    current: Option<usize>,
}

impl TrueTypeProgram {
    pub fn try_new(data: &[u8]) -> Result<Self> {
        let mut reader = TrueTypeReader::new(data);
        let sfnt = Sfnt::try_new(&mut reader)?;

        if let Some(cmap_record) = sfnt.get_table("cmap") {
            let offset = cmap_record.offset();
            reader.reset_offset(offset as usize);
            let _version = reader.read_be_u16()?;
            let num_tables = reader.read_be_u16()?;
            let mut cmap_records = Vec::new();
            for _i in 0..num_tables {
                let platform_id = reader.read_be_u16()?;
                let encoding_id = reader.read_be_u16()?;
                let sub_offset = offset + reader.read_be_u32()?;
                cmap_records.push((platform_id, encoding_id, sub_offset));
            }

            let mut cmaps = Vec::new();
            for (platform_id, encoding_id, offset) in cmap_records {
                let cmap = TCmap::try_new(platform_id, encoding_id, offset, &mut reader)?;
                cmaps.push(cmap);
            }

            return Ok(TrueTypeProgram {
                cmaps,
                current: None,
            });
        } else {
            return Ok(TrueTypeProgram {
                cmaps: Vec::new(),
                current: None,
            });
        }
    }
    pub fn num_cmap(&self) -> usize {
        self.cmaps.len()
    }
    pub fn get_cmap(&self, platform_id: u16, encoding_id: u16) -> Option<&TCmap> {
        for cmap in self.cmaps.iter() {
            if cmap.platform_id() == platform_id && cmap.encoding_id() == encoding_id {
                return Some(cmap);
            }
        }
        None
    }
    pub fn first_cmap(&self) -> Option<&TCmap> {
        self.cmaps.first()
    }

    pub fn selct_charmap(&mut self, is_symbolic: bool) {
        if is_symbolic {
            for (i, cmap) in self.cmaps.iter().enumerate() {
                if cmap.platform_id() == 3 && cmap.encoding_id() == 1 {
                    self.current = Some(i);
                    return;
                }
            }
        }
        for (i, cmap) in self.cmaps.iter().enumerate() {
            if cmap.platform_id() == 3 && cmap.encoding_id() == 0 {
                self.current = Some(i);
                return;
            }
        }

        for (i, cmap) in self.cmaps.iter().enumerate() {
            if cmap.platform_id() == 1 && cmap.encoding_id() == 0 {
                self.current = Some(i);
                return;
            }
        }
        if self.cmaps.len() > 0 {
            self.current = Some(0);
        }
    }

    pub fn current_cmap(&self) -> Option<&TCmap> {
        if let Some(index) = self.current {
            return self.cmaps.get(index);
        }
        None
    }

    pub fn has_unicode_cmap(&self) -> bool {
        for cmap in self.cmaps.iter() {
            if cmap.platform_id() == 3 && cmap.encoding_id() == 1 {
                return true;
            }
        }
        false
    }
    pub fn has_mac_roman_cmap(&self) -> bool {
        for cmap in self.cmaps.iter() {
            if cmap.platform_id() == 1 && cmap.encoding_id() == 0 {
                return true;
            }
        }
        false
    }
    pub fn has_mac_unicode_cmap(&self) -> bool {
        for cmap in self.cmaps.iter() {
            if cmap.platform_id() == 0 && cmap.encoding_id() < 4 {
                return true;
            }
        }
        false
    }

    pub fn has_ms_symbol_cmap(&self) -> bool {
        for cmap in self.cmaps.iter() {
            if cmap.platform_id() == 3 && cmap.encoding_id() == 0 {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::font::builtin_font::load_builtin_font_data;

    use super::TrueTypeProgram;

    #[test]
    fn test_truetype_program() {
        let font_data = load_builtin_font_data("TimesNewRoman").unwrap();
        let program = TrueTypeProgram::try_new(font_data).unwrap();
        println!("{:?}", program);
    }
}
