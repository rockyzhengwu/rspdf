// https://adobe-type-tools.github.io/font-terch-notes/pdfs/5014.CIDFont_Spec.pdf
// Operators That Use CIDs as Selectors
// functional: specifies range of CIDFont characters
//   begincidchar endcidchar && begincidrange endcidrange
//
// Operators That Use Character Names
// or Character Codes as Selectors
//

// this implement combine "cid" font cmap and charcode to  unicode cmap

use std::io::{Read, Seek};

use encoding_rs::UTF_16BE;

use crate::errors::{PDFError, PDFResult};
use crate::font::cmap::predefined::get_predefine_cmap;
use crate::font::cmap::{CMap, CidRange, CodeSpaceRange};
use crate::object::{PDFName, PDFNumber, PDFObject, PDFString};
use crate::parser::character_set::hex_to_u8;
use crate::parser::syntax::{SyntaxParser, Token};

pub struct CMapParser<T: Seek + Read> {
    syntax_parser: SyntaxParser<T>,
}

fn is_comment(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[..2] == [b'%', b'%']
}

pub fn hex_bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut res: u32 = 0;
    for h in bytes {
        let n = hex_to_u8(h) as u32;
        res = res * 16 + n;
    }
    res
}

fn incresement(bytes: &mut Vec<u8>, pos: usize) {
    if pos > 0 && bytes[pos] == 255 {
        bytes[pos] = 0;
        incresement(bytes, pos - 1);
    } else {
        bytes[pos] += 1;
    }
}

impl<T: Seek + Read> CMapParser<T> {
    pub fn new(syntax_parser: SyntaxParser<T>) -> Self {
        CMapParser { syntax_parser }
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

    fn process_code_space_range(&mut self, cmap: &mut CMap) -> PDFResult<()> {
        loop {
            if self
                .syntax_parser
                .check_next_token(&Token::new_other("endcodespacerange"))?
            {
                return Ok(());
            }
            let _start_hex = self.syntax_parser.next_token()?;
            let low = self.syntax_parser.read_hex_string()?;
            let _start_hex = self.syntax_parser.next_token()?;
            let high = self.syntax_parser.read_hex_string()?;

            let char_size = high.bytes().len() as u8;
            let high_value: u32 = hex_bytes_to_u32(high.bytes());
            let low_value: u32 = hex_bytes_to_u32(low.bytes());

            cmap.add_code_space_range(CodeSpaceRange::new(char_size / 2, low_value, high_value));
        }
    }

    fn process_cid_range(&mut self, cmap: &mut CMap) -> PDFResult<()> {
        loop {
            if self
                .syntax_parser
                .check_next_token(&Token::new_other("endcidrange"))?
            {
                return Ok(());
            }
            let start = self.read_whole_hex()?;
            let end = self.read_whole_hex()?;
            let val = self.get_cid()?;
            let range = CidRange::new(
                start.binary_bytes()?.as_slice(),
                end.binary_bytes()?.as_slice(),
                val,
            );
            cmap.add_cid_range(range);
        }
    }

    fn get_cid(&mut self) -> PDFResult<u32> {
        let val = match self.syntax_parser.read_object()? {
            PDFObject::String(s) => hex_bytes_to_u32(s.bytes()),
            PDFObject::Number(n) => n.as_u32(),
            _ => {
                return Err(PDFError::FontCmapFailure(
                    "parse cmap cidrange val not a number{:?} ".to_string(),
                ));
            }
        };
        Ok(val)
    }

    fn process_cid_char(&mut self, cmap: &mut CMap) -> PDFResult<()> {
        loop {
            if self
                .syntax_parser
                .check_next_token(&Token::new_other("endcidchar"))?
            {
                return Ok(());
            }
            let key = self.read_whole_hex()?;
            let val = self.get_cid()?;
            cmap.add_cid(key.binary_bytes()?.as_slice(), val);
        }
    }

    fn process_bf_range(&mut self, cmap: &mut CMap) -> PDFResult<()> {
        loop {
            if self
                .syntax_parser
                .check_next_token(&Token::new_other("endbfrange"))?
            {
                return Ok(());
            }
            let start = self.read_whole_hex()?;
            let end = self.read_whole_hex()?;
            let obj = self.syntax_parser.read_object()?;
            match obj {
                PDFObject::Arrray(arr) => {
                    let mut bytes = start.binary_bytes()?;
                    let pos = bytes.len() - 1;
                    for v in arr {
                        let vs: PDFString = v.try_into()?;
                        let val_bytes = vs.binary_bytes()?;
                        let (s, _, has_err) = UTF_16BE.decode(val_bytes.as_slice());
                        if has_err {
                            return Err(PDFError::FontCmapFailure(
                                "cmap decode bfrange error".to_string(),
                            ));
                        }
                        cmap.add_unicode(start.bytes(), s.to_string());
                        incresement(&mut bytes, pos);
                    }
                }
                PDFObject::String(s) => {
                    let n = hex_bytes_to_u32(end.bytes()) - hex_bytes_to_u32(start.bytes()) + 1;
                    let mut val_bytes = s.binary_bytes()?;
                    let mut code_bytes = start.binary_bytes()?;
                    for _ in 0..n {
                        let (s, _, has_err) = UTF_16BE.decode(&val_bytes);
                        if has_err {
                            return Err(PDFError::FontCmapFailure(
                                "cmap decode bfrange error".to_string(),
                            ));
                        }
                        cmap.add_unicode(code_bytes.as_slice(), s.to_string());
                        let pos = val_bytes.len() - 1;
                        incresement(&mut val_bytes, pos);
                        let code_pos = code_bytes.len() - 1;
                        incresement(&mut code_bytes, code_pos);
                    }
                }
                _ => {}
            }
        }
    }

    fn process_bf_char(&mut self, cmap: &mut CMap) -> PDFResult<()> {
        loop {
            if self
                .syntax_parser
                .check_next_token(&Token::new_other("endbfchar"))?
            {
                return Ok(());
            }
            let key = self.read_whole_hex()?;
            let val = self.read_whole_hex()?;
            let bv = val.binary_bytes()?;
            let (s, _, has_err) = UTF_16BE.decode(bv.as_slice());
            if has_err {
                return Err(PDFError::FontCmapFailure(
                    "cmap decode bfrange error".to_string(),
                ));
            }
            let kbyte = key.binary_bytes()?;
            cmap.add_unicode(kbyte.as_slice(), s.to_string());
        }
    }

    pub fn parse(&mut self) -> PDFResult<CMap> {
        let mut cmap = CMap::default();
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
                        b"begincodespacerange" => self.process_code_space_range(&mut cmap)?,
                        b"begincidrange" => self.process_cid_range(&mut cmap)?,
                        b"begincidchar" => self.process_cid_char(&mut cmap)?,
                        b"beginbfrange" => self.process_bf_range(&mut cmap)?,
                        b"beginbfchar" => self.process_bf_char(&mut cmap)?,
                        b"usecmap" => {
                            if let Some(PDFObject::Name(name)) = objs.pop() {
                                let other_cmap = get_predefine_cmap(name.name()).ok_or(
                                    PDFError::FontCmapFailure(format!(
                                        "{} cmap not found",
                                        name.name()
                                    )),
                                )?;
                                cmap.usecmap(other_cmap);
                            }
                            objs.clear();
                        }
                        b"endcmap" => {
                            break;
                        }
                        b"def" => {
                            if objs.len() < 2 {
                                objs.clear();
                                continue;
                            }
                            let val = objs.pop().unwrap();
                            let key = objs.pop().unwrap();
                            if let PDFObject::Name(name) = key {
                                match name.name() {
                                    "CMapName" => {
                                        cmap.set_name(val.to_owned().as_string()?);
                                    }
                                    "CMapType" => {
                                        cmap.set_type(Some(val.to_owned().as_u32()? as u8));
                                    }
                                    "WMode" => {
                                        cmap.set_wmdoe(Some(val.to_owned().as_u32()? as u8));
                                    }
                                    _ => {}
                                }
                            }
                            objs.clear();
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

#[cfg(test)]
mod tests {

    use crate::font::cmap::parser::hex_bytes_to_u32;
    use crate::font::cmap::CMap;

    #[test]
    fn test_cmap_parse() {
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
        let cmap = CMap::new_from_bytes(content.as_slice()).unwrap();
        assert_eq!(cmap.name(), "AAAAAA+F4+0");
        assert_eq!(cmap.cmap_type(), Some(2));
    }

    #[test]
    fn test_usecmap() {
        let bytes = include_bytes!("../../../cmaps/Identity-H");
        let cmap = CMap::new_from_bytes(bytes.as_slice()).unwrap();
        println!("{:?}", cmap);
    }
    #[test]
    fn test_hex_to_u32() {
        let bytes = b"ff";
        let value = hex_bytes_to_u32(bytes);
        assert_eq!(value, 255);
        let bytes = b"0f";
        let value = hex_bytes_to_u32(bytes);
        assert_eq!(value, 15);

        let bytes = b"d862dde7";
        let value = hex_bytes_to_u32(bytes);
        println!("{:?}", value);
    }
}
