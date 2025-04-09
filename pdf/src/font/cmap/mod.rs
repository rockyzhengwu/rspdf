use std::collections::HashMap;

use crate::{error::PdfError, font::CharCode};
use font_data::cmap::get_predefine_cmap_data;
use parser::CmapParser;

use crate::error::Result;

mod parser;

#[derive(Debug, Clone, Default)]
pub struct CodeSpaceRange {
    size: u8,
    low: u32,
    high: u32,
}

impl CodeSpaceRange {
    pub fn new(low: u32, high: u32, size: u8) -> Self {
        Self { low, high, size }
    }

    pub fn is_contain_code(&self, code: u32) -> bool {
        self.low <= code && self.high >= code
    }

    pub fn size(&self) -> &u8 {
        &self.size
    }
}

#[derive(Debug, Clone, Default)]
pub struct CidRange {
    start: u32,
    end: u32,
    start_cid: u32,
    length: u8,
}

#[derive(Debug, Clone, Default)]
pub struct Cmap {
    name: String,
    wmode: Option<u8>,
    cmap_type: Option<u8>,
    code_space_range: Vec<CodeSpaceRange>,
    cid_to_unicode: HashMap<u32, String>,
    code_to_cids: HashMap<u32, u32>,
}

impl Cmap {
    pub fn try_new(data: Vec<u8>) -> Result<Self> {
        let parser = CmapParser::new(data);
        parser.parse()
    }

    pub fn new_from_predefined(name: &str) -> Result<Self> {
        let cmap_data = get_predefine_cmap_data(name).ok_or(PdfError::Font(format!(
            "predefine cmap {:?} not found",
            name
        )))?;
        let cmap = Cmap::try_new(cmap_data.to_vec())?;
        Ok(cmap)
    }

    pub fn usecmap(&mut self, other: Cmap) {
        self.code_space_range = other.code_space_range;
        self.cid_to_unicode = other.cid_to_unicode;
        self.code_to_cids = other.code_to_cids;
    }

    pub fn add_code_space_range(&mut self, space_range: CodeSpaceRange) {
        self.code_space_range.push(space_range);
    }

    pub fn add_unicode(&mut self, src_code: u32, unicode: String) {
        self.cid_to_unicode.insert(src_code, unicode);
    }

    pub fn unicode(&self, char: &CharCode) -> Option<&str> {
        return self.cid_to_unicode.get(&char.code).map(|v| v.as_str());
    }

    pub fn add_code_to_cid(&mut self, code: u32, cid: u32) {
        self.code_to_cids.insert(code, cid);
    }

    pub fn wmode(&self) -> Option<u8> {
        self.wmode
    }

    fn find_charsize(&self, bytes: &[u8]) -> Option<u8> {
        let code = bytes_to_u32(bytes);
        let size = bytes.len() as u8;
        for range in &self.code_space_range {
            if range.size != size {
                continue;
            }
            if range.is_contain_code(code) {
                return Some(range.size);
            }
        }
        None
    }

    pub fn next_char(&self, bytes: &[u8], offset: usize) -> Option<CharCode> {
        if offset > bytes.len() {
            return None;
        }
        let mut codecs = Vec::new();
        for i in 0..4 {
            if offset + i > bytes.len() {
                return None;
            }
            codecs.push(bytes[offset + i].to_owned());
            if let Some(v) = self.find_charsize(codecs.as_slice()) {
                let ch = CharCode::new(bytes_to_u32(codecs.as_slice()), v, 0.0);
                return Some(ch);
            }
        }
        None
    }

    pub fn chars(&self, bytes: &[u8]) -> Vec<CharCode> {
        let mut res = Vec::new();
        let mut offset = 0;
        while offset < bytes.len() {
            if let Some(charcode) = self.next_char(bytes, offset) {
                offset += charcode.length() as usize;
                res.push(charcode);
            }
        }
        res
    }
}

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut res = 0;
    for v in bytes {
        res = (res << 8) + v.to_owned() as u32;
    }
    res
}
