use std::collections::HashMap;
use std::io::Cursor;

use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFName, PDFNumber, PDFObject, PDFString};
use crate::page::operation::{to_command, Operation};
use crate::parser::syntax::{SyntaxParser, Token};

pub struct ContentParser {
    syntax_parser: SyntaxParser<Cursor<Vec<u8>>>,
}

impl ContentParser {
    pub fn try_new(content: Vec<u8>) -> PDFResult<Self> {
        //println!(
        //    "{:#?}",
        //    String::from_utf8_lossy(content.as_slice()).to_string()
        //);
        let cursor = Cursor::new(content);
        let syntax_parser = SyntaxParser::try_new(cursor)?;
        Ok(ContentParser { syntax_parser })
    }

    pub fn parse_operation(&mut self) -> PDFResult<Operation> {
        let mut params: Vec<PDFObject> = Vec::new();
        loop {
            let obj = self.read_object()?;
            match obj {
                PDFObject::String(ref s) => {
                    if let Some(cmd) = to_command(s.bytes()) {
                        let op = Operation::new(cmd, params);
                        return Ok(op);
                    } else {
                        params.push(obj);
                    }
                }
                _ => {
                    params.push(obj);
                }
            }
        }
    }

    pub fn read_object(&mut self) -> PDFResult<PDFObject> {
        let token = self.syntax_parser.next_token()?;

        match token {
            Token::Number(_) => Ok(PDFObject::Number(PDFNumber::Real(token.to_f64()?))),
            Token::StartHexString => {
                let hex = self.syntax_parser.read_hex_string()?;
                Ok(PDFObject::String(hex))
            }
            Token::StartLiteralString => {
                let s = self.syntax_parser.read_literal_string()?;
                Ok(PDFObject::String(s))
            }
            Token::StartArray => {
                let mut objs = Vec::new();
                while !self.syntax_parser.check_next_token(&Token::EndArray)? {
                    let obj = self.read_object()?;
                    objs.push(obj);
                }
                Ok(PDFObject::Arrray(objs))
            }
            Token::StartDict => {
                let mut dict = HashMap::new();
                while !self.syntax_parser.check_next_token(&Token::EndDict)? {
                    let keyword = self.syntax_parser.next_token()?;
                    let obj = self.read_object()?;
                    dict.insert(keyword.to_string()?, obj);
                }
                Ok(PDFObject::Dictionary(dict))
            }
            Token::Name(_) => Ok(PDFObject::Name(PDFName::new(&token.to_string()?))),
            Token::Other(bytes) => Ok(PDFObject::String(PDFString::Literial(bytes))),
            Token::Eof => {
                // TODO
                Ok(PDFObject::Null)
            }
            _ => Err(PDFError::InvalidSyntax(format!(
                "invalid token:{:?}",
                token
            ))),
        }
    }
}

impl Iterator for ContentParser {
    type Item = Operation;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
