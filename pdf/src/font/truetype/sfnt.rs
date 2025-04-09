use std::collections::HashMap;

use super::reader::TrueTypeReader;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct TableRecord {
    tag: String,
    check_sum: u32,
    offset: u32,
    length: u32,
}

impl TableRecord {
    pub fn try_new(reader: &mut TrueTypeReader) -> Result<Self> {
        let tag = reader.read_tag()?;
        let check_sum = reader.read_be_u32()?;
        let offset = reader.read_be_u32()?;
        let length = reader.read_be_u32()?;
        Ok(Self {
            tag,
            check_sum,
            offset,
            length,
        })
    }
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

#[derive(Debug, Clone)]
pub struct Sfnt {
    tables: HashMap<String, TableRecord>,
}
impl Sfnt {
    pub fn try_new(reader: &mut TrueTypeReader) -> Result<Self> {
        let _version = reader.read_be_u32()?;
        let num_tables = reader.read_be_u16()?;
        let search_range = reader.read_be_u16()?;
        let _entry_selector = reader.read_be_u16()?;
        let range_shift = reader.read_be_u16()?;
        println!("{:?}", num_tables);
        if num_tables == 0 {
            return Ok(Sfnt {
                tables: HashMap::new(),
            });
        }
        assert_eq!(range_shift, 16 * num_tables - search_range);
        let mut tables = HashMap::new();
        for _ in 0..num_tables {
            let record = TableRecord::try_new(reader)?;
            tables.insert(record.tag.clone(), record);
        }
        Ok(Sfnt { tables })
    }
    pub fn get_table(&self, name: &str) -> Option<&TableRecord> {
        self.tables.get(name)
    }
}
