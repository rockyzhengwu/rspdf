use std::collections::HashMap;
use std::io::Cursor;
use std::u8;

use crate::font::cmap::predefined::get_predefine_cmap_ref;
use crate::lexer::Tokenizer;
use crate::object::PDFString;

pub mod parser;
pub mod predefined;

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
    usecmap: Option<String>,
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

    pub fn set_usecmap(&mut self, other: String) {
        self.usecmap = Some(other);
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

    pub fn code_to_cid(&self, bytes: &[u8]) -> Vec<u32> {
        let mut cids: Vec<u32> = Vec::new();
        if matches!(self.name.as_str(), "Identity-H" | "Identity-V") {
            for v in bytes.chunks(2) {
                if v.len() == 2 {
                    let h = v[0] as u32;
                    let l = v[1] as u32;
                    let code = (h << 8) + l;
                    let cid = match self.code_to_cid.get(&code) {
                        Some(cd) => cd,
                        // TODO fix
                        None => match self.usecmap {
                            Some(ref scp) => {
                                let um = get_predefine_cmap_ref(scp);
                                let ucid = um.code_to_cid.get(&code).unwrap();
                                ucid
                            }
                            None => {
                                panic!("usep is none");
                            }
                        },
                    };
                    cids.push(cid.to_owned())
                } else {
                    cids.push(v[0] as u32)
                }
            }
        }
        cids
    }

    pub fn cid_to_string(&self, cids: &[u32]) -> String {
        let mut res = String::new();
        for c in cids {
            if let Some(ch) = self.code_to_character.get(c) {
                res.push(char::from_u32(ch.to_owned()).unwrap());
            }
        }
        res
    }
}
