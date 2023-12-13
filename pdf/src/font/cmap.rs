use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};

use crate::errors::{PDFError, PDFResult};
use crate::lexer::Tokenizer;
use crate::object::{PDFName, PDFNumber, PDFObject, PDFString};
use crate::token::Token;

#[derive(Default, Clone, Debug)]
pub struct CMap {
    code_to_unicode: HashMap<u32, char>,
}

impl CMap {
    pub fn decode_string(&self, content: &PDFString) -> String {
        let mut res = String::new();
        for b in content.bytes() {
            let code = *b as u32;
            if let Some(c) = self.code_to_unicode.get(&code) {
                res.push(c.to_owned());
            }
        }
        res
    }
    pub fn is_empty(&self) -> bool {
        self.code_to_unicode.is_empty()
    }
}

struct CMapParser<T: Seek + Read> {
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
    fn new(tokenizer: Tokenizer<T>) -> Self {
        CMapParser { tokenizer }
    }

    pub fn parse(&mut self) -> PDFResult<CMap> {
        let mut code_to_unicode: HashMap<u32, char> = HashMap::new();

        while !self.tokenizer.check_next(&Token::PDFEof)? {
            let mut command = self.parse_cmp_command()?;
            // TODO fix
            let cmd: String = command
                .pop()
                .ok_or(PDFError::FontCmapFailure(
                    "Command object is empty".to_string(),
                ))?
                .as_string()?;

            match cmd.as_str() {
                "def" => {
                    if command.len() == 2 {
                        let _key = command[0].as_string()?;
                        let _val = command[1].to_owned();
                        //println!("defaine {:?},{:?}", key, val);
                    }
                }
                "endcidchar" | "endbfchar" => {
                    if command.len() >= 2 {
                        for item in command.chunks(2) {
                            let mark = hex_to_number(&item[0])?;
                            // TODO unwrap
                            let c = char::from_u32(hex_to_number(&item[1])?).unwrap();
                            code_to_unicode.insert(mark, c);
                        }
                        // TODO
                    }
                }
                "endcidrange" | "endbfrange" => {
                    if command.len() >= 3 {
                        for item in command.chunks(3) {
                            //println!("{:?}", item);
                            let start = hex_to_number(&item[0])?;
                            let end = hex_to_number(&item[1])?;
                            match item[2] {
                                PDFObject::String(_) => {
                                    let mut iv = hex_to_number(&item[2])?;
                                    for m in start..=end {
                                        let c = char::from_u32(iv).unwrap();
                                        code_to_unicode.insert(m, c);
                                        iv += 1;
                                    }
                                }
                                PDFObject::Arrray(_) => {}
                                _ => {
                                    return Err(PDFError::FontCmapFailure(
                                        "parse cmaps endbfrange error".to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
                "usecmap" => {
                    //TODO,
                }
                "endcodespacerange" => {
                    //TODO
                }
                _ => {}
            }
        }
        Ok(CMap { code_to_unicode })
    }

    fn parse_cmp_command(&mut self) -> PDFResult<Vec<PDFObject>> {
        let mut command = Vec::new();
        while !self.tokenizer.check_next(&Token::PDFOther(Vec::new()))? {
            let obj = self.parse_obj()?;
            command.push(obj);
        }
        let token = self.tokenizer.next_token()?;
        match token {
            Token::PDFOther(ref v) => {
                command.push(PDFObject::String(PDFString::Literial(v.to_owned())));
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
            Token::PDFEof => Ok(PDFObject::Null),
            _ => Err(PDFError::FontCmapFailure(format!(
                "parse Cmap object Unexpected Token:{:?}",
                token
            ))),
        }
    }

    fn read_dict(&mut self) -> PDFResult<PDFObject> {
        let mut dict = HashMap::new();
        while !self.tokenizer.check_next(&Token::PDFCloseDict)? {
            let key: PDFName = self.parse_obj()?.try_into()?;
            let val = self.parse_obj()?;
            dict.insert(key, val);
        }
        self.tokenizer.next_token()?;
        Ok(PDFObject::Dictionary(dict))
    }

    fn read_array(&mut self) -> PDFResult<PDFObject> {
        let mut array = Vec::new();
        while !self.tokenizer.check_next(&Token::PDFCloseArray)? {
            let val = self.parse_obj()?;
            array.push(val);
        }
        self.tokenizer.next_token()?;
        Ok(PDFObject::Arrray(array))
    }
}

impl CMap {
    pub fn new(buffer: &[u8]) -> Self {
        let cursor = Cursor::new(buffer);
        let tokenizer = Tokenizer::new(cursor);
        let mut parser = CMapParser::new(tokenizer);
        parser.parse().unwrap()
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
        let cmap = CMap::new(content.as_slice());
        assert_eq!(cmap.code_to_unicode.len(), 107);
    }
}
