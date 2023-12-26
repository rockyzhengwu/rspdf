// https://adobe-type-tools.github.io/font-tech-notes/pdfs/5014.CIDFont_Spec.pdf
// Operators That Use CIDs as Selectors
// functional: specifies range of CIDFont characters
//   begincidchar endcidchar && begincidrange endcidrange
//
// Operators That Use Character Names
// or Character Codes as Selectors
//

use std::collections::HashMap;
use std::io::{Read, Seek};

use log::warn;

use crate::errors::{PDFError, PDFResult};
use crate::font::cmap::{CMap, CodeSpaceRange};
use crate::lexer::Tokenizer;
use crate::object::{PDFName, PDFNumber, PDFObject, PDFString};
use crate::token::Token;

pub struct CMapParser<T: Seek + Read> {
    tokenizer: Tokenizer<T>,
}

fn hex_to_number(o: &PDFObject) -> PDFResult<u32> {
    match o {
        PDFObject::String(PDFString::HexString(bytes)) => {
            u32::from_str_radix(String::from_utf8_lossy(bytes).to_string().as_str(), 16).map_err(
                |_| PDFError::FontCmapFailure(format!("hex bytes {:?} can't convert to number", o)),
            )
        }
        _ => Err(PDFError::FontCmapFailure(format!(
            "hex_to_number need HexString got:{:?}",
            o
        ))),
    }
}

impl<T: Seek + Read> CMapParser<T> {
    pub fn new(tokenizer: Tokenizer<T>) -> Self {
        CMapParser { tokenizer }
    }

    pub fn parse(&mut self) -> PDFResult<CMap> {
        let mut cmap = CMap::default();

        while !self.tokenizer.check_next_type(&Token::PDFEof)? {
            let mut command = self.parse_cmp_command()?;
            // TODO fix
            if command.len() < 2 {
                continue;
            }
            let cmd: String = command
                .pop()
                .ok_or(PDFError::FontCmapFailure(
                    "Command object is empty".to_string(),
                ))?
                .as_string()?;
            match cmd.as_str() {
                "def" => {
                    if command.len() == 2 {
                        let key = command[0].as_string()?;
                        match key.as_str() {
                            "CMapName" => {
                                let val = command[1].to_owned().as_string()?;
                                cmap.set_name(val);
                            }
                            "CMapType" => {
                                let val = command[1].to_owned().as_i64().unwrap() as u8;
                                cmap.set_type(Some(val));
                            }
                            _ => {
                                //
                            }
                        }
                    }
                }
                "endcidchar" => {
                    if command.len() >= 2 {
                        for item in command.chunks(2) {
                            let mark = hex_to_number(&item[0])?;
                            let uv = hex_to_number(&item[1])?;
                            cmap.add_cid(mark, uv);
                        }
                    } else {
                        warn!("Cmap parer endcidchar command not valid");
                    }
                }
                "endbfchar" => {
                    if command.len() >= 2 {
                        for item in command.chunks(2) {
                            let mark = hex_to_number(&item[0])?;
                            let uv = hex_to_number(&item[1])?;
                            cmap.add_character(mark, uv);
                        }
                    } else {
                        warn!("Cmap parer endbfchar command not valid");
                    }
                }
                "endcidrange" => {
                    if command.len() >= 3 {
                        for item in command.chunks(3) {
                            let start = hex_to_number(&item[0])?;
                            let end = hex_to_number(&item[1])?;
                            match item[2] {
                                PDFObject::String(_) => {
                                    let iv = hex_to_number(&item[2])?;
                                    cmap.add_range_cid(start, end, iv);
                                }
                                PDFObject::Number(_) => {
                                    let n = item[2].as_i64().unwrap() as u32;
                                    cmap.add_range_cid(start, end, n)
                                }
                                PDFObject::Arrray(_) => {
                                    unimplemented!()
                                }
                                _ => {
                                    return Err(PDFError::FontCmapFailure(format!(
                                        "parse cmaps endbfrange error expected String or Array:{:?}",
                                        item[2]
                                    )));
                                }
                            }
                        }
                    }
                }
                "endbfrange" => {
                    if command.len() >= 3 {
                        for item in command.chunks(3) {
                            let start = hex_to_number(&item[0])?;
                            let end = hex_to_number(&item[1])?;
                            match item[2] {
                                PDFObject::String(_) => {
                                    let iv = hex_to_number(&item[2])?;
                                    cmap.add_range_to_character(start, end, iv)
                                }
                                PDFObject::Number(_) => {
                                    let n = item[2].as_i64().unwrap() as u32;
                                    cmap.add_range_to_character(start, end, n);
                                }
                                PDFObject::Arrray(_) => {
                                    unimplemented!()
                                }
                                _ => {
                                    return Err(PDFError::FontCmapFailure(format!(
                                        "parse cmaps endbfrange error expected String or Array:{:?}",
                                        item[2]
                                    )));
                                }
                            }
                        }
                    }
                }
                "usecmap" => {
                    let name = command[0].as_string().unwrap();
                    cmap.set_usecmap(name);
                }
                "endcodespacerange" => {
                    let low = hex_to_number(&command[0])?;
                    let high = hex_to_number(&command[1])?;
                    cmap.add_code_space_range(CodeSpaceRange { low, high })
                }
                _ => {}
            }
        }
        Ok(cmap)
    }

    fn parse_cmp_command(&mut self) -> PDFResult<Vec<PDFObject>> {
        let mut command = Vec::new();
        while !self
            .tokenizer
            .check_next_type(&Token::PDFOther(Vec::new()))?
            && !self.tokenizer.check_next_type(&Token::PDFEof)?
        {
            let obj = self.parse_obj()?;
            command.push(obj);
        }
        let token = self.tokenizer.next_token()?;
        match token {
            Token::PDFOther(ref v) => {
                command.push(PDFObject::String(PDFString::Literial(v.to_owned())));
            }
            Token::PDFEof => {
                return Ok(command);
            }
            _ => {
                panic!("unexpect cmap toekn ");
            }
        }
        Ok(command)
    }

    fn parse_obj(&mut self) -> PDFResult<PDFObject> {
        let token = self.tokenizer.next_token()?;
        match token {
            Token::PDFOpenDict => Ok(self.read_dict()?),
            Token::PDFOpenArray => Ok(self.read_array()?),
            Token::PDFName(ref name) => Ok(PDFObject::Name(PDFName::new(name.as_str()))),
            Token::PDFNumber(n) => Ok(PDFObject::Number(PDFNumber::Integer(n))),
            Token::PDFReal(r) => Ok(PDFObject::Number(PDFNumber::Real(r))),
            Token::PDFHexString(ref hex) => {
                Ok(PDFObject::String(PDFString::HexString(hex.to_owned())))
            }
            Token::PDFLiteralString(ref s) => {
                Ok(PDFObject::String(PDFString::Literial(s.to_owned())))
            }
            Token::PDFOther(s) => Ok(PDFObject::String(PDFString::Literial(s))),
            _ => Err(PDFError::FontCmapFailure(format!(
                "parse Cmap object Unexpected Token:{:?}",
                token
            ))),
        }
    }

    fn read_dict(&mut self) -> PDFResult<PDFObject> {
        let mut dict = HashMap::new();
        while !self.tokenizer.check_next_type(&Token::PDFCloseDict)? {
            let key: PDFName = self.parse_obj()?.try_into()?;
            let val = self.parse_obj()?;
            dict.insert(key, val);
        }
        self.tokenizer.next_token()?;
        Ok(PDFObject::Dictionary(dict))
    }

    fn read_array(&mut self) -> PDFResult<PDFObject> {
        let mut array = Vec::new();
        while !self.tokenizer.check_next_type(&Token::PDFCloseArray)? {
            let val = self.parse_obj()?;
            array.push(val);
        }
        self.tokenizer.next_token()?;
        Ok(PDFObject::Arrray(array))
    }
}

#[cfg(test)]
mod tests {

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
        let cmap = CMap::new_from_bytes(content.as_slice());
        assert_eq!(cmap.code_to_character_len(), 107);
        assert_eq!(cmap.name(), "AAAAAA+F4+0");
        assert_eq!(cmap.cmap_type(), Some(2));
    }

    #[test]
    fn test_usecmap() {
        let bytes = include_bytes!("../../../cmaps/Identity-V");
        let cmap = CMap::new_from_bytes(bytes.as_slice());
        println!("{:?}", cmap);
    }
}
