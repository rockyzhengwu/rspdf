use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{Cursor, Read, Seek};

use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFName, PDFNumber, PDFObject, PDFString};
use crate::parser::character_set::hex_to_u8;
use crate::parser::syntax::{SyntaxParser, Token};

#[derive(Default, Debug)]
pub struct ToUnicodeMap {
    mutil_map: HashMap<u32, HashSet<u32>>,
}

impl ToUnicodeMap {
    pub fn insert_value(&mut self, charcode: u32, unicode: u32) {
        if let Some(set) = self.mutil_map.get_mut(&charcode) {
            set.insert(unicode);
        } else {
            let mut set = HashSet::new();
            set.insert(unicode);
            self.mutil_map.insert(charcode, set);
        }
    }
}

pub struct ToUnicodeParser<T: Seek + Read> {
    syntax_parser: SyntaxParser<T>,
}

fn is_comment(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[..2] == [b'%', b'%']
}

fn hex_bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut res: u32 = 0;
    for h in bytes {
        let n = hex_to_u8(h) as u32;
        res = res * 16 + n;
    }
    res
}
impl<T: Seek + Read> ToUnicodeParser<T> {
    pub fn new(syntax_parser: SyntaxParser<T>) -> Self {
        ToUnicodeParser { syntax_parser }
    }

    fn read_whole_hex(&mut self) -> PDFResult<PDFString> {
        let obj = self.syntax_parser.read_object()?;
        match obj {
            PDFObject::String(s) => Ok(s),
            _ => Err(PDFError::FontCmapFailure(format!(
                "cmap parser read hex string error:{:?}",
                obj
            ))),
        }
    }

    fn process_bf_range(&mut self, tounicode: &mut ToUnicodeMap) -> PDFResult<()> {
        loop {
            if self
                .syntax_parser
                .check_next_token(&Token::new_other("endbfrange"))?
            {
                return Ok(());
            }
            let start = hex_bytes_to_u32(self.read_whole_hex()?.bytes());
            let end = hex_bytes_to_u32(self.read_whole_hex()?.bytes());
            match self.syntax_parser.read_object()? {
                PDFObject::String(s) => {
                    let mut val = hex_bytes_to_u32(s.bytes());
                    let mut code = start;
                    while code < end {
                        tounicode.insert_value(code, val);
                        code += 1;
                        val += 1;
                    }
                }
                PDFObject::Number(n) => {
                    let mut val = n.as_u32();
                    let mut code = start;
                    while code < end {
                        tounicode.insert_value(code, val);
                        code += 1;
                        val += 1;
                    }
                }
                PDFObject::Arrray(arr) => {
                    let mut code = start;
                    for obj in arr {
                        let val = hex_bytes_to_u32(&obj.bytes()?);
                        tounicode.insert_value(code, val);
                        code += 1;
                    }
                }
                _ => {
                    return Err(PDFError::FontCmapFailure(
                        "parse cmap bfrange val not a number{:?} ".to_string(),
                    ))
                }
            }
        }
    }
    fn process_bf_char(&mut self, tounicode: &mut ToUnicodeMap) -> PDFResult<()> {
        loop {
            if self
                .syntax_parser
                .check_next_token(&Token::new_other("endbfchar"))?
            {
                return Ok(());
            }
            let key = hex_bytes_to_u32(self.read_whole_hex()?.bytes());
            let val = hex_bytes_to_u32(self.read_whole_hex()?.bytes());
            tounicode.insert_value(key, val)
        }
    }

    pub fn parse(&mut self) -> PDFResult<ToUnicodeMap> {
        let mut cmap = ToUnicodeMap::default();
        let mut objs: Vec<PDFObject> = Vec::new();
        loop {
            let token = self.syntax_parser.next_token()?;
            match token {
                Token::Other(bytes) => {
                    if is_comment(&bytes) {
                        self.syntax_parser.move_next_line()?;
                        continue;
                    }
                    match bytes.as_slice() {
                        b"beginbfrange" => self.process_bf_range(&mut cmap)?,
                        b"beginbfchar" => self.process_bf_char(&mut cmap)?,
                        b"endcmap" => {
                            break;
                        }
                        _ => {}
                    }
                }
                Token::StartHexString => {
                    let s = self.syntax_parser.read_hex_string()?;
                    objs.push(PDFObject::String(s));
                }
                Token::StartLiteralString => {
                    let s = self.syntax_parser.read_literal_string()?;
                    objs.push(PDFObject::String(s));
                }
                Token::StartDict => {
                    let d = self.syntax_parser.read_object()?;
                    objs.push(d);
                }
                Token::Number(_) => {
                    let n = PDFObject::Number(PDFNumber::Real(token.to_f64()?));
                    objs.push(n);
                }
                Token::Name(_) => {
                    let name = PDFObject::Name(PDFName::new(token.to_string()?.as_str()));
                    objs.push(name);
                }
                Token::Eof => {
                    break;
                }
                _ => objs.clear(),
            }
        }
        Ok(cmap)
    }
}

impl ToUnicodeMap {
    pub fn new_from_bytes(buffer: &[u8]) -> PDFResult<Self> {
        let cursor = Cursor::new(buffer);
        let syntax = SyntaxParser::try_new(cursor)?;
        let mut parser = ToUnicodeParser::new(syntax);
        parser.parse()
    }
}

#[cfg(test)]
mod tests {
    use crate::font::to_unicode::ToUnicodeMap;

    #[test]
    fn test_tounicode_parse() {
        let content = b"
/CIDInit /ProcSet findresource begin 12 dict begin begincmap /CIDSystemInfo <<
/Registry (AAAAAA+F4+0) /Ordering (T1UV) /Supplement 0 >> def
/CMapName /AAAAAA+F4+0 def
/CMapType 2 def
1 begincodespacerange <18> <fc> endcodespacerange
15 beginbfchar
<18> <02D8>
<19> <02C7>
<21> <0021>
<5d> <005D>
<5f> <005F>
<84> <2014>
<85> <2013>
<b8> <00B8>
<e4> <00E4>
<e9> <00E9>
<ed> <00ED>
<ef> <00EF>
<f4> <00F4>
<f6> <00F6>
<fc> <00FC>
endbfchar
9 beginbfrange
<23> <26> <0023>
<28> <3b> <0028>
<3d> <3f> <003D>
<41> <5b> <0041>
<61> <7e> <0061>
<8d> <8e> <201C>
<8f> <90> <2018>
<93> <94> <FB01>
<e0> <e1> <00E0>
endbfrange
endcmap CMapName currentdict /CMap defineresource pop end end";
        let tounicode = ToUnicodeMap::new_from_bytes(content.as_slice()).unwrap();
    }
}
