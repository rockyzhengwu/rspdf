use std::collections::HashMap;
use std::io::Cursor;

use log::warn;

use crate::errors::PDFResult;
use crate::parser::syntax::SyntaxParser;

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
        if bytes.len() == 2 {
            self.code_to_unicode_one.insert(charcode, ch);
        } else if bytes.len() == 4 {
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

    #[allow(dead_code)]
    pub fn wmode(&self) -> Option<u8> {
        self.wmode
    }

    pub fn charcode_to_unicode(&self, charcode: &u32) -> Option<&str> {
        if charcode < &256 {
            self.code_to_unicode_one.get(charcode).map(|s| s.as_str())
        } else {
            self.code_to_unicode_two.get(charcode).map(|s| s.as_str())
        }
    }

    pub fn charcodes_to_unicode(&self, bytes: &[u8]) -> Vec<char> {
        //if self.code_to_unicode.is_empty() {
        //    let n = bytes.len();
        //    return vec![char::REPLACEMENT_CHARACTER; n];
        //}
        //let mut res = Vec::new();
        //if self.code_space_range.is_empty() {
        //    for b in bytes {
        //        let code = *b as u32;
        //        let u = self.charcode_to_unicode(&code);
        //        res.push(u);
        //    }
        //    return res;
        //}
        //let mut code = 0;
        //let mut n = 0;
        //for b in bytes.iter() {
        //    code += *b as u32;
        //    n += 1;
        //    if self.find_charsize(code, n).is_some() {
        //        res.push(self.charcode_to_unicode(&code));
        //        code = 0;
        //        n = 0;
        //        continue;
        //    }
        //    if n == 4 {
        //        res.push(self.charcode_to_unicode(&code));
        //        n = 0;
        //        code = 0;
        //    }
        //}
        //res
        unimplemented!()
    }

    pub fn charcode_to_cid(&self, charcode: &u32) -> u32 {
        //match self.code_to_cid.get(charcode) {
        //    Some(cid) => cid.to_owned(),
        //    None => match self.usecmap {
        //        Some(ref scp) => {
        //            let nm = get_predefine_cmap(scp);
        //            nm.charcode_to_cid(charcode)
        //        }
        //        None => {
        //            panic!("faild map charcode to cid");
        //        }
        //    },
        //}
        unimplemented!()
    }

    fn find_charsize(&self, code: u32, size: u8) -> Option<u8> {
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

    pub fn charcodes_to_cid(&self, bytes: &[u8]) -> Vec<u32> {
        //pass
        let mut code = 0;
        let mut res = Vec::new();
        let mut n = 0;
        for b in bytes.iter() {
            code += *b as u32;
            n += 1;
            if self.find_charsize(code, n).is_some() {
                res.push(self.charcode_to_cid(&code));
                code = 0;
                continue;
            }
            if n == 4 {
                res.push(self.charcode_to_cid(&code));
                n = 0;
                code = 0;
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
