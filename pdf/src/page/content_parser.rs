use std::collections::HashMap;

use crate::error::{PdfError, Result};
use crate::object::dictionary::PdfDict;
use crate::object::string::PdfLiteral;
use crate::object::PdfObject;
use crate::page::operator::{is_command, Operator};
use crate::reader::{PdfReader, Token};

pub struct ContentParser {
    reader: PdfReader,
}

impl ContentParser {
    pub fn new(content: Vec<u8>) -> Self {
        let reader = PdfReader::new(content);
        Self { reader }
    }

    pub fn read_operator(&self) -> Result<Operator> {
        let mut operands: Vec<PdfObject> = Vec::new();
        loop {
            self.reader.skip_white_space()?;
            if self.reader.is_eof() {
                return Err(PdfError::ContentParser("Eof".to_string()));
            }
            let token = self.reader.peek_token()?;
            match token {
                Token::Other(buf) => {
                    if is_command(buf) {
                        if buf == b"BI" {
                            let t = self.reader.read_token()?;
                            let mut image_info = HashMap::new();
                            loop {
                                let token = self.reader.peek_token()?;
                                match token {
                                    Token::StartName => {
                                        self.reader.read_token()?;
                                        let key = self.reader.read_name()?;
                                        self.reader.skip_white_space()?;
                                        let val = self.reader.read_object()?;
                                        image_info.insert(key.name().to_string(), val);
                                    }
                                    Token::Other(buf) => {
                                        if buf == b"ID" {
                                            let t = self.reader.read_token()?;
                                            assert!(t.is_other_key(b"ID"));
                                            let _white_space = self.reader.read_byte();
                                            //self.reader.skip_white_space()?;
                                            let bytes =
                                                self.reader.read_until_reach(b"\nEI").unwrap();
                                            let image =
                                                PdfObject::LiteralString(PdfLiteral::new(bytes));
                                            operands
                                                .push(PdfObject::Dict(PdfDict::new(image_info)));
                                            operands.push(image);
                                            break;
                                        } else {
                                            return Err(PdfError::ContentParser(
                                                "Inline iamge get unexpeced key ".to_string(),
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(PdfError::ContentParser(
                                            "Inline iamge get unexpeced key ".to_string(),
                                        ))
                                    }
                                }
                            }
                            return Ok(Operator::new("EI".to_string(), operands));
                        } else {
                            let op = String::from_utf8(buf.to_vec()).unwrap();
                            self.reader.read_token()?;
                            return Ok(Operator::new(op, operands));
                        }
                    } else {
                        return Err(PdfError::ContentParser(format!(
                            "ContentParser unexpected token:{:?}",
                            token
                        )));
                    }
                }
                _ => {
                    let obj = self.reader.read_object()?;
                    operands.push(obj);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::page::content_parser::ContentParser;

    #[test]
    fn test_content_parser() {
        let content: &[u8] = &[
            113, 10, 47, 82, 101, 108, 97, 116, 105, 118, 101, 67, 111, 108, 111, 114, 105, 109,
            101, 116, 114, 105, 99, 32, 114, 105, 32, 10, 47, 71, 83, 50, 32, 103, 115, 10, 66, 84,
            10, 47, 70, 49, 32, 49, 32, 84, 102, 10, 49, 51, 46, 57, 49, 56, 51, 32, 48, 32, 48,
            32, 49, 51, 46, 57, 49, 56, 51, 32, 50, 50, 49, 46, 54, 53, 57, 32, 53, 50, 51, 46, 57,
            54, 54, 49, 32, 84, 109, 10, 47, 67, 115, 56, 32, 99, 115, 32, 49, 32, 115, 99, 110,
            10, 48, 46, 48, 54, 57, 56, 32, 84, 99, 10, 48, 32, 84, 119, 10, 91, 40, 70, 111, 117,
            114, 116, 104, 41, 45, 51, 54, 49, 46, 55, 40, 69, 100, 105, 116, 105, 111, 110, 41,
            93, 84, 74, 10, 47, 70, 50, 32, 49, 32, 84, 102, 10, 50, 56, 46, 51, 51, 51, 55, 32,
            48, 32, 48, 32, 50, 56, 46, 51, 51, 51, 55, 32, 50, 50, 49, 46, 54, 53, 57, 32, 52, 56,
            49, 46, 51, 50, 52, 51, 32, 84, 109, 10, 48, 32, 84, 99, 10, 91, 40, 68, 97, 116, 97,
            41, 45, 50, 52, 48, 46, 49, 40, 83, 116, 114, 117, 99, 116, 117, 114, 101, 115, 41, 93,
            84, 74, 10, 48, 32, 45, 49, 46, 48, 53, 50, 55, 32, 84, 68, 10, 45, 48, 46, 48, 48, 48,
            49, 32, 84, 99, 10, 91, 40, 97, 110, 100, 41, 45, 50, 52, 48, 46, 50, 40, 65, 108, 103,
            111, 114, 105, 116, 104, 109, 41, 93, 84, 74, 10, 84, 42, 10, 48, 32, 84, 99, 10, 91,
            40, 65, 110, 97, 108, 121, 115, 105, 115, 41, 45, 50, 52, 48, 46, 51, 40, 105, 110, 41,
            93, 84, 74, 10, 56, 53, 46, 50, 57, 57, 50, 32, 48, 32, 48, 32, 56, 53, 46, 50, 57, 57,
            50, 32, 50, 56, 48, 46, 51, 49, 52, 32, 51, 53, 49, 46, 48, 49, 56, 55, 32, 84, 109,
            10, 40, 67, 41, 84, 106, 10, 52, 55, 46, 55, 49, 57, 56, 32, 48, 32, 48, 32, 52, 55,
            46, 55, 49, 57, 56, 32, 51, 50, 54, 46, 49, 49, 55, 50, 32, 51, 54, 55, 46, 57, 50, 49,
            32, 84, 109, 10, 40, 43, 43, 41, 84, 106, 10, 69, 84, 10, 81, 10,
        ];
        let parser = ContentParser::new(content.to_vec());
        loop {
            let op = parser.read_operator();
            println!("OP:{:?}", op);
            if op.is_err() {
                break;
            }
        }
    }
}
