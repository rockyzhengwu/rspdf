use crate::error::Result;
use crate::font::truetype::reader::TrueTypeReader;

#[derive(Debug, Clone)]
pub struct Format0Cmap {
    gids: [u8; 256],
}

impl Format0Cmap {
    pub fn try_new(reader: &mut TrueTypeReader) -> Result<Self> {
        let mut gids = Vec::new();
        for _ in 0..256 {
            let gid = reader.read_u8()?;
            gids.push(gid);
        }
        Ok(Format0Cmap {
            gids: gids.try_into().unwrap(),
        })
    }
    pub fn get_gid(&self, c: u8) -> u8 {
        self.gids[c as usize]
    }
}

#[derive(Debug, Clone)]
pub struct Format4Cmap {
    seg_count_x2: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
    end_code: Vec<u16>,
    reserved_pad: u16,
    start_code: Vec<u16>,
    id_delta: Vec<i16>,
    id_range_offset: Vec<u16>,
    glyph_id_array: Vec<u16>,
}
impl Format4Cmap {
    pub fn try_new(reader: &mut TrueTypeReader, length: u16) -> Result<Self> {
        let start = reader.offset();
        let seg_count_x2 = reader.read_be_u16()?;
        let search_range = reader.read_be_u16()?;
        let entry_selector = reader.read_be_u16()?;
        let range_shift = reader.read_be_u16()?;
        let mut end_code = Vec::new();
        for _i in 0..seg_count_x2 / 2 {
            let v = reader.read_be_u16()?;
            end_code.push(v)
        }
        let reserved_pad = reader.read_be_u16()?;
        let mut start_code = Vec::new();
        for _i in 0..seg_count_x2 / 2 {
            let v = reader.read_be_u16()?;
            start_code.push(v);
        }
        let mut id_delta = Vec::new();
        for _i in 0..seg_count_x2 / 2 {
            let v = reader.read_be_i16()?;
            id_delta.push(v);
        }
        let mut id_range_offset = Vec::new();
        for _i in 0..seg_count_x2 / 2 {
            let v = reader.read_be_u16()?;
            id_range_offset.push(v);
        }
        let end = reader.offset();
        let remain_len = length as usize - (end - start);
        let mut glyph_id_array = Vec::new();
        for _i in 0..remain_len / 2 {
            let v = reader.read_be_u16()?;
            glyph_id_array.push(v);
        }
        Ok(Self {
            seg_count_x2,
            search_range,
            entry_selector,
            range_shift,
            end_code,
            reserved_pad,
            start_code,
            id_delta,
            id_range_offset,
            glyph_id_array,
        })
    }

    pub fn get_gid(&self, c: u16) -> u16 {
        for i in 0..(self.seg_count_x2 / 2) as usize {
            let start = self.start_code[i];
            let end = self.end_code[i];
            if c >= start && c <= end {
                if self.id_range_offset[i] == 0 {
                    let gid = (c as i32 + self.id_delta[i] as i32) as u16;
                    return gid;
                } else {
                    let glyph_index_address = self.id_range_offset[i] / 2 + (c - start) + i as u16
                        - self.id_range_offset.len() as u16;
                    let gid = self.glyph_id_array[glyph_index_address as usize];
                    return gid;
                }
            }
        }
        return 0;
    }
}

#[derive(Debug, Clone)]
pub enum SubCmap {
    F0(Format0Cmap),
    F4(Format4Cmap),
}

#[derive(Debug, Clone)]
pub struct TCmap {
    platform_id: u16,
    encoding_id: u16,
    offset: u32,
    submap: SubCmap,
}
impl TCmap {
    pub fn try_new(
        platform_id: u16,
        encoding_id: u16,
        offset: u32,
        reader: &mut TrueTypeReader,
    ) -> Result<Self> {
        reader.reset_offset(offset as usize);
        let format = reader.read_be_u16()?;
        let length = reader.read_be_u16()?;
        let _language = reader.read_be_u16()?;
        match format {
            0 => {
                let sb = Format0Cmap::try_new(reader)?;
                let submap = SubCmap::F0(sb);
                return Ok(TCmap {
                    platform_id,
                    encoding_id,
                    offset,
                    submap,
                });
            }
            4 => {
                let sb = Format4Cmap::try_new(reader, length)?;
                let submap = SubCmap::F4(sb);
                return Ok(TCmap {
                    platform_id,
                    encoding_id,
                    offset,
                    submap,
                });
            }
            _ => {
                panic!("TrueTypeProgram format: {:?}", format);
            }
        }
    }

    pub fn get_gid(&self, charcode: u16) -> u16 {
        match &self.submap {
            SubCmap::F0(f) => {
                return f.get_gid(charcode as u8) as u16;
            }
            SubCmap::F4(f) => {
                return f.get_gid(charcode);
            }
        }
    }

    pub fn platform_id(&self) -> u16 {
        self.platform_id
    }

    pub fn encoding_id(&self) -> u16 {
        self.encoding_id
    }
}
