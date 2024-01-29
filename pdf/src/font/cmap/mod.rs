use core::panic;
use std::collections::HashMap;
use std::io::Cursor;
use std::u8;

use log::warn;

use crate::errors::PDFResult;
use crate::font::cmap::predefined::get_predefine_cmap;
use crate::parser::syntax::SyntaxParser;

pub mod parser;
pub mod predefined;

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct CodeSpaceRange {
    char_size: u8,
    low: u32,
    high: u32,
}




#[allow(dead_code)]
impl CodeSpaceRange {
    pub fn new(char_size: u8, low: u32, high: u32) -> Self {
        CodeSpaceRange {
            char_size,
            low,
            high,
        }
    }
    pub fn is_contain_code(&self, code: u32) -> bool {
        self.low <= code && self.high >= code
    }

    pub fn char_size(&self) -> &u8 {
        &self.char_size
    }
}

#[derive(Default, Clone, Debug)]
pub struct CMap {
    name: String,
    wmode: Option<u8>,
    cmap_type: Option<u8>,
    usecmap: Option<String>,
    code_space_range: Vec<CodeSpaceRange>,
    code_to_character: HashMap<u32, u32>,
    code_to_cid: HashMap<u32, u32>,
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

    pub fn add_range_cid(&mut self, low: u32, high: u32, start: u32) {
        for code in low..=high {
            let cid = start + code - low;
            self.add_cid(code, cid)
        }
    }

    pub fn add_range_to_character(&mut self, low: u32, high: u32, start: u32) {
        for code in low..=high {
            let ch = start + code - low;
            self.add_character(code, ch);
        }
    }

    #[allow(dead_code)]
    pub fn cmap_type(&self) -> Option<u8> {
        self.cmap_type
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn add_character(&mut self, code: u32, ch: u32) {
        self.code_to_character.insert(code, ch);
    }

    pub fn add_cid(&mut self, code: u32, cid: u32) {
        self.code_to_cid.insert(code, cid);
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

    pub fn charcode_to_unicode(&self, charcode: &u32) -> char {
        if let Some(c) = self.code_to_character.get(charcode) {
            char::from_u32(c.to_owned()).unwrap()
        } else {
            warn!("cid to unicode not found: {:?}", charcode);
            ' '
        }
    }

    pub fn charcodes_to_unicode(&self, bytes: &[u8]) -> Vec<char> {
        if self.code_to_character.is_empty() {
            let n = bytes.len();
            return vec![char::REPLACEMENT_CHARACTER; n];
        }
        let mut res = Vec::new();
        if self.code_space_range.is_empty() {
            for b in bytes {
                let code = *b as u32;
                let u = self.charcode_to_unicode(&code);
                res.push(u);
            }
            return res;
        }
        let mut code = 0;
        let mut n = 0;
        for b in bytes.iter() {
            code += *b as u32;
            n += 1;
            if self.find_charsize(code, n).is_some() {
                res.push(self.charcode_to_unicode(&code));
                code = 0;
                n = 0;
                continue;
            }
            if n == 4 {
                res.push(self.charcode_to_unicode(&code));
                n = 0;
                code = 0;
            }
        }
        res
    }

    pub fn charcode_to_cid(&self, charcode: &u32) -> u32 {
        match self.code_to_cid.get(charcode) {
            Some(cid) => cid.to_owned(),
            None => match self.usecmap {
                Some(ref scp) => {
                    let nm = get_predefine_cmap(scp);
                    nm.charcode_to_cid(charcode)
                }
                None => {
                    panic!("faild map charcode to cid");
                }
            },
        }
    }

    fn find_charsize(&self, code: u32, size: u8) -> Option<u8> {
        for range in &self.code_space_range {
            if range.char_size != size {
                continue;
            }
            if range.is_contain_code(code) {
                return Some(range.char_size);
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
