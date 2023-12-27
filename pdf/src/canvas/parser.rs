use std::io::{Read, Seek};

use crate::canvas::operation::Operation;
use crate::errors::{PDFError, PDFResult};
use crate::lexer::Tokenizer;
use crate::object::{PDFArray, PDFDictionary, PDFName, PDFNumber, PDFObject, PDFString};
use crate::token::Token;

pub(crate) struct CanvasParser<T: Seek + Read> {
    tokenizer: Tokenizer<T>,
}

impl<T: Seek + Read> CanvasParser<T> {
    pub fn new(tokenizer: Tokenizer<T>) -> Self {
        CanvasParser { tokenizer }
    }

    pub fn parse_op(&mut self) -> PDFResult<Operation> {
        let mut objs = Vec::new();
        // BI
        if self
            .tokenizer
            .check_next_value(&Token::PDFOther(vec![66, 73]))?
        {
            return self.parse_inline_image();
        }
        while !self
            .tokenizer
            .check_next_type(&Token::PDFOther(Vec::new()))?
        {
            let obj = self.read_object()?;
            objs.push(obj);
        }
        let token = self.tokenizer.next_token()?;
        let op = token.as_string()?;
        Ok(Operation::new(op, objs))
    }

    pub fn parse_inline_image(&mut self) -> PDFResult<Operation> {
        let _bi = self.tokenizer.next_token();
        let mut objs = Vec::new();
        while !self
            .tokenizer
            .check_next_value(&Token::PDFOther(vec![73, 68]))?
        {
            let obj = self.read_object()?;
            objs.push(obj);
        }
        let _id = self.tokenizer.next_token()?;
        let image_buffer = self.tokenizer.read_unitil(b"\nEI")?;
        objs.push(PDFObject::String(PDFString::Literial(image_buffer)));
        Ok(Operation::new("EI".to_string(), objs))
    }

    pub fn read_object(&mut self) -> PDFResult<PDFObject> {
        let token = self.tokenizer.next_token()?;
        match token {
            Token::PDFOpenDict => self.read_dict(),
            Token::PDFOpenArray => self.read_array(),
            Token::PDFReal(v) => Ok(PDFObject::Number(PDFNumber::Real(v))),
            Token::PDFNumber(v) => Ok(PDFObject::Number(PDFNumber::Integer(v))),
            Token::PDFLiteralString(s) => Ok(PDFObject::String(PDFString::Literial(s))),
            Token::PDFHexString(s) => Ok(PDFObject::String(PDFString::HexString(s))),
            Token::PDFName(n) => Ok(PDFObject::Name(PDFName::new(n.as_str()))),
            Token::PDFOther(v) => Ok(PDFObject::String(PDFString::Literial(v))),
            _ => Err(PDFError::InvalidContentSyntax(format!(
                "{:?} not a content object stater",
                token
            ))),
        }
    }
    pub fn read_dict(&mut self) -> PDFResult<PDFObject> {
        let token = self.tokenizer.next_token()?;
        let mut dict = PDFDictionary::default();
        loop {
            match token {
                Token::PDFEof | Token::PDFCloseDict => break,
                Token::PDFName(ref name) => {
                    let key = PDFName::new(name);
                    let val = self.read_object()?;
                    dict.insert(key, val);
                }
                _ => {
                    return Err(PDFError::InvalidContentSyntax(format!(
                        "{:?} can't b Dictionary Key",
                        token
                    )))
                }
            }
        }
        Ok(PDFObject::Dictionary(dict))
    }

    pub fn read_array(&mut self) -> PDFResult<PDFObject> {
        let mut array = PDFArray::default();
        while !self.tokenizer.check_next_type(&Token::PDFCloseArray)? {
            let val = self.read_object()?;
            array.push(val);
        }
        self.tokenizer.next_token()?;
        Ok(PDFObject::Arrray(array))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Cursor;
    use std::path::PathBuf;

    use super::CanvasParser;
    use crate::lexer::Tokenizer;
    fn peek_filename(name: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push(format!("tests/resources/{}", name));
        d
    }

    #[test]
    fn test_parser() {
        let contents =
            "[(Ta)-80.2(i)-80.1(p)-81(e)-80.2(i)-965.4(T)20.4(o)-80.7(k)-80.9(y)-80.5(o)]TJ";
        let cursor = Cursor::new(contents);
        let tokenizer = Tokenizer::new(cursor);
        let mut parser = CanvasParser::new(tokenizer);
        let op = parser.parse_op();
        println!("{:?}", op);
    }

    #[test]
    fn test_inline_image() {
        let filename = peek_filename("content_with_inline_image.bin");
        let file = File::open(filename).unwrap();
        let tokenizer = Tokenizer::new(file);
        let mut parser = CanvasParser::new(tokenizer);
        // TODO assert
        while let Ok(op) = parser.parse_op() {
            println!("{:?}", op);
        }
    }
    #[test]
    fn test_tmp() {
        let filename = "/home/zhengwu/workspace/private/rspdf/content.bin";
        let file = File::open(filename).unwrap();
        let tokenizer = Tokenizer::new(file);
        let mut parser = CanvasParser::new(tokenizer);
        // TODO assert
        while let Ok(op) = parser.parse_op() {
            println!("{:?}", op);
        }
    }
}
