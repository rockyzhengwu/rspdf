use std::collections::HashMap;
use std::io::Cursor;
use std::u8;

use crate::lexer::Tokenizer;
use crate::object::PDFString;

pub mod parser;

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct CodeSpaceRange {
    low: u32,
    high: u32,
}

#[allow(dead_code)]
impl CodeSpaceRange {
    pub fn is_contain_code(&self, code: u32) -> bool {
        self.low <= code && self.high >= code
    }
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct CMap {
    name: String,
    wmode: Option<u8>,
    cmap_type: Option<u8>,
    code_space_range: Vec<CodeSpaceRange>,
    code_to_character: HashMap<u32, u32>,
    code_to_cid: HashMap<u32, u32>,
}

impl CMap {
    pub fn code_to_character_len(&self) -> usize {
        self.code_to_character.len()
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn cmap_type(&self) -> Option<u8> {
        self.cmap_type
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_type(&mut self, cmap_type: Option<u8>) {
        self.cmap_type = cmap_type;
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

    pub fn add_character(&mut self, code: u32, ch: u32) {
        self.code_to_character.insert(code, ch);
    }

    pub fn add_cid(&mut self, code: u32, cid: u32) {
        self.code_to_cid.insert(code, cid);
    }

    pub fn new_from_bytes(buffer: &[u8]) -> Self {
        let cursor = Cursor::new(buffer);
        let tokenizer = Tokenizer::new(cursor);
        let mut parser = parser::CMapParser::new(tokenizer);
        parser.parse().unwrap()
    }

    pub fn has_to_unicode(&self) -> bool {
        self.code_to_character.is_empty()
    }

    pub fn decode_string(&self, content: &PDFString) -> String {
        let mut res = String::new();
        for b in content.bytes() {
            let code = *b as u32;
            if let Some(c) = self.code_to_character.get(&code) {
                res.push(char::from_u32(c.to_owned()).unwrap());
            }
        }
        res
    }
}
