use std::collections::HashMap;
use std::io::Cursor;

use log::warn;

use crate::errors::PDFResult;
use crate::font::cmap::charcode::CharCode;
use crate::parser::syntax::SyntaxParser;

pub mod charcode;
pub mod parser;
pub mod predefined;

#[derive(Default, Clone, Debug)]
pub struct CodeSpaceRange {
    size: u8,
    low: u32,
    high: u32,
}
impl CodeSpaceRange {
    pub fn new(size: u8, low: u32, high: u32) -> Self {
        CodeSpaceRange { size, low, high }
    }
    pub fn is_contain_code(&self, code: u32) -> bool {
        self.low <= code && self.high >= code
    }

    pub fn size(&self) -> &u8 {
        &self.size
    }
}

#[derive(Default, Clone, Debug)]
pub struct CidRange {
    start: u32,
    end: u32,
    start_cid: u32,
    length: u8,
}
impl CidRange {
    fn new(start: &[u8], end: &[u8], start_cid: u32) -> Self {
        let length = start.len() as u8;
        let start = bytes_to_u32(start);
        let end = bytes_to_u32(end);
        CidRange {
            start,
            end,
            start_cid,
            length,
        }
    }
    fn find_cide(&self, bytes: &[u8]) -> Option<u32> {
        let l = bytes.len() as u8;
        let v = bytes_to_u32(bytes);
        if l != self.length || v < self.start || v > self.end {
            return None;
        }
        Some(self.start_cid + v - self.start)
    }
}

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut res = 0;
    for v in bytes {
        res = (res << 8) + v.to_owned() as u32;
    }
    res
}

#[derive(Default, Clone, Debug)]
pub struct CMap {
    name: String,
    wmode: Option<u8>,
    cmap_type: Option<u8>,
    usecmap: Option<String>,
    code_space_range: Vec<CodeSpaceRange>,
    code_to_unicode_one: HashMap<u32, String>,
    code_to_unicode_two: HashMap<u32, String>,
    code_to_cid: HashMap<u8, HashMap<u32, u32>>,
    cid_ranges: Vec<CidRange>,
}

impl CMap {
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_usecmap(&mut self, other: String) {
        self.usecmap = Some(other);
    }

    pub fn set_type(&mut self, cmap_type: Option<u8>) {
        self.cmap_type = cmap_type;
    }

    pub fn set_wmdoe(&mut self, wmode: Option<u8>) {
        self.wmode = wmode;
    }

    pub fn add_code_space_range(&mut self, space_range: CodeSpaceRange) {
        self.code_space_range.push(space_range);
    }

    pub fn cmap_type(&self) -> Option<u8> {
        self.cmap_type
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn add_unicode(&mut self, bytes: &[u8], ch: String) {
        let charcode = bytes_to_u32(bytes);
        if bytes.len() == 1 {
            self.code_to_unicode_one.insert(charcode, ch);
        } else if bytes.len() == 2 {
            self.code_to_unicode_two.insert(charcode, ch);
        }
    }

    pub fn add_simple_unicode(&mut self, code: u32, ch: String) {
        self.code_to_unicode_one.insert(code, ch);
    }

    pub fn add_cid(&mut self, code: &[u8], cid: u32) {
        let len = code.len() as u8;
        if let Some(map) = self.code_to_cid.get_mut(&len) {
            let key = bytes_to_u32(code);
            map.insert(key, cid);
        }
    }

    pub fn add_cid_range(&mut self, range: CidRange) {
        self.cid_ranges.push(range);
    }

    pub fn new_from_bytes(buffer: &[u8]) -> PDFResult<Self> {
        let cursor = Cursor::new(buffer);
        let syntax = SyntaxParser::try_new(cursor)?;
        let mut parser = parser::CMapParser::new(syntax);
        parser.parse()
    }

    pub fn wmode(&self) -> Option<u8> {
        self.wmode
    }

    pub fn charcode_to_unicode(&self, bytes: &[u8]) -> Option<&str> {
        let n = bytes.len();
        match n {
            1 => self
                .code_to_unicode_one
                .get(&bytes_to_u32(bytes))
                .map(|s| s.as_str()),
            2 => self
                .code_to_unicode_two
                .get(&bytes_to_u32(bytes))
                .map(|s| s.as_str()),
            _ => None,
        }
    }

    pub fn has_unicode_map(&self) -> bool {
        !self.code_to_unicode_two.is_empty() || !self.code_to_unicode_one.is_empty()
    }

    pub fn charcodes_to_unicode(&self, bytes: &[u8]) -> Vec<String> {
        if !self.has_unicode_map() {
            let n = bytes.len();
            return vec![String::from(char::REPLACEMENT_CHARACTER); n];
        }
        let mut res: Vec<String> = Vec::new();
        if self.code_space_range.is_empty() {
            for b in bytes {
                let code = vec![b.to_owned()];
                if let Some(s) = self.charcode_to_unicode(code.as_slice()) {
                    res.push(s.to_string());
                }
            }
            return res;
        }
        let mut charcode: Vec<u8> = Vec::new();
        for b in bytes.iter() {
            charcode.push(b.to_owned());
            if self.find_charsize(charcode.as_slice()).is_some() {
                if let Some(s) = self.charcode_to_unicode(charcode.as_slice()) {
                    res.push(s.to_string());
                } else {
                    warn!("notfound char:{:?}", charcode);
                }
                charcode.clear();
                continue;
            }
            if charcode.len() == 4 {
                if let Some(v) = self.charcode_to_unicode(charcode.as_slice()) {
                    res.push(v.to_string());
                }
                charcode.clear();
            }
        }
        res
    }

    pub fn charcode_to_cid(&self, charcode: &[u8]) -> Option<u32> {
        let l = charcode.len() as u8;
        if let Some(map) = self.code_to_cid.get(&l) {
            let v = bytes_to_u32(charcode);
            if let Some(cid) = map.get(&v) {
                return Some(cid.to_owned());
            }
        }

        for range in &self.cid_ranges {
            if let Some(v) = range.find_cide(charcode) {
                return Some(v);
            }
        }
        None
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
                let ch = CharCode::new(bytes_to_u32(codecs.as_slice()), v);
                return Some(ch);
            }
        }
        None
    }

    pub fn charcode_to_cids(&self, bytes: &[u8]) -> Vec<u32> {
        //pass
        let mut res = Vec::new();
        let mut charcode: Vec<u8> = Vec::new();
        for b in bytes.iter() {
            charcode.push(b.to_owned());
            if self.find_charsize(charcode.as_slice()).is_some() {
                if let Some(v) = self.charcode_to_cid(charcode.as_slice()) {
                    res.push(v);
                }
                charcode.clear();
                continue;
            }
            if charcode.len() == 4 {
                // TODO fix this unwrap
                res.push(self.charcode_to_cid(charcode.as_slice()).unwrap());
                charcode.clear()
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hex_value() {
        let bytes: &[u8] = b"ffff";
        println!("value {:?}", bytes);
    }
}
