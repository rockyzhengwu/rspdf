use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

use log::warn;

use crate::errors::{PDFError, PDFResult};
use crate::object::{
    PDFDictionary, PDFIndirect, PDFName, PDFNumber, PDFObject, PDFStream, PDFString,
};
use crate::parser::character_set::{
    buf_to_number, buf_to_real, hex_to_u8, is_delimiter, is_end_of_line, is_number, is_regular,
    is_whitespace, is_xdigit,
};

enum StringStatus {
    Normal,
    Backslash,
    Octal,
    FinishOctal,
    CarriageReturn,
}

#[derive(Debug)]
pub struct SyntaxParser<T: Seek + Read> {
    stream: T,
    size: u64,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Other(Vec<u8>),
    Number(Vec<u8>),
    Name(Vec<u8>),
    StartHexString,
    EndHexString,
    StartLiteralString,
    EndLiteralString,
    StartArray,
    EndArray,
    StartDict,
    EndDict,
    Eof,
}

impl Token {
    pub fn new_other(content: &str) -> Token {
        Token::Other(content.bytes().collect())
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Token::Number(_))
    }

    pub fn to_f64(&self) -> PDFResult<f64> {
        match self {
            Token::Number(bytes) => Ok(buf_to_real(bytes)),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} faild covnert to f64",
                self
            ))),
        }
    }

    pub fn to_i64(&self) -> PDFResult<i64> {
        match self {
            Token::Number(bytes) => Ok(buf_to_number(bytes)),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} faild convert to i64",
                self
            ))),
        }
    }

    pub fn to_u32(&self) -> PDFResult<u32> {
        match self {
            Token::Number(bytes) => Ok(buf_to_number(bytes) as u32),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} faild convert to u32",
                self
            ))),
        }
    }

    pub fn to_u16(&self) -> PDFResult<u16> {
        match self {
            Token::Number(bytes) => Ok(buf_to_number(bytes) as u16),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} faild convert to u16",
                self
            ))),
        }
    }

    pub fn is_other_value(&self, expected: &[u8]) -> bool {
        match self {
            Token::Other(bytes) => bytes == expected,
            _ => false,
        }
    }

    pub fn to_string(&self) -> PDFResult<String> {
        match self {
            Token::Name(bytes) => String::from_utf8(bytes.clone()).map_err(|e| {
                PDFError::TokenConvertFailure(format!("{:?} token faild convert to string", e))
            }),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} not Name token",
                self
            ))),
        }
    }
}

impl<T: Seek + Read> SyntaxParser<T> {
    pub fn try_new(mut stream: T) -> PDFResult<Self> {
        let size = stream.seek(SeekFrom::End(0)).map_err(|e| {
            PDFError::InvalidSyntax(format!("input stream not seek failed :{:?}", e))
        })?;
        stream
            .rewind()
            .map_err(|e| PDFError::InvalidSyntax(format!("input stream failed rewind:{:?}", e)))?;

        Ok(SyntaxParser { stream, size })
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn move_next_line(&mut self) -> PDFResult<()> {
        let mut ch = self.read_next_char()?;
        loop {
            if ch == b'\n' {
                break;
            }
            if ch == b'\r' {
                ch = self.read_next_char()?;
                if ch == b'\n' {
                    break;
                } else {
                    self.step_back()?;
                }
            }
            ch = self.read_next_char()?;
        }
        Ok(())
    }

    pub fn read_hex_string(&mut self) -> PDFResult<PDFString> {
        let mut ch = self.read_next_char()?;
        let mut bytes = Vec::new();
        loop {
            if ch == b'>' {
                break;
            }
            bytes.push(ch);
            ch = self.read_next_char()?
        }
        Ok(PDFString::HexString(bytes))
    }

    pub fn read_literal_string(&mut self) -> PDFResult<PDFString> {
        let mut nest_level: i32 = 0;
        let mut status: StringStatus = StringStatus::Normal;
        let mut ch = self.read_next_char()?;
        let mut bytes = Vec::new();
        let mut esc_octal: u8 = 0;
        loop {
            match status {
                StringStatus::Normal => match ch {
                    b'(' => nest_level += 1,
                    b')' => {
                        if nest_level == 0 {
                            return Ok(PDFString::Literial(bytes));
                        }
                        nest_level -= 1;
                    }
                    b'\\' => {
                        status = StringStatus::Backslash;
                    }
                    _ => {
                        bytes.push(ch);
                    }
                },
                StringStatus::Backslash => {
                    match ch {
                        b'0'..=b'7' => {
                            status = StringStatus::Octal;
                            esc_octal = ch - b'0';
                        }
                        b'\r' => status = StringStatus::CarriageReturn,
                        b'n' => {
                            status = StringStatus::Normal;
                            bytes.push(b'\n')
                        }
                        b'r' => {
                            status = StringStatus::Normal;
                            bytes.push(b'\r')
                        }
                        b't' => {
                            status = StringStatus::Normal;
                            bytes.push(b'\t')
                        }
                        b'b' => {
                            status = StringStatus::Normal;
                            bytes.push(8)
                        }
                        b'f' => {
                            status = StringStatus::Normal;
                            bytes.push(12)
                        }
                        b'\n' => {
                            status = StringStatus::Normal;
                        } //donothing
                        _ => {
                            status = StringStatus::Normal;
                            bytes.push(ch)
                        }
                    }
                }
                StringStatus::Octal => match ch {
                    b'0'..=b'7' => {
                        esc_octal = esc_octal * 8 + ch - b'0';
                        status = StringStatus::FinishOctal;
                    }
                    _ => {
                        bytes.push(esc_octal);
                        status = StringStatus::Normal;
                    }
                },
                StringStatus::FinishOctal => {
                    status = StringStatus::Normal;
                    match ch {
                        b'0'..=b'7' => {
                            esc_octal = esc_octal * 8 + ch - b'0';
                            bytes.push(esc_octal);
                        }
                        _ => {
                            bytes.push(esc_octal);
                        }
                    }
                }
                StringStatus::CarriageReturn => {
                    status = StringStatus::Normal;
                    if ch != b'\n' {
                        continue;
                    }
                }
            }
            ch = self.read_next_char()?;
        }
    }

    pub fn read_object(&mut self) -> PDFResult<PDFObject> {
        let token = self.next_token()?;
        match token {
            Token::Number(_) => {
                let pos = self.current_position()?;
                let token_2 = self.next_token()?;
                if !token_2.is_number() {
                    self.seek_to(pos)?;
                    return Ok(PDFObject::Number(PDFNumber::Real(token.to_f64()?)));
                }
                let token_3 = self.next_token()?;
                if token_3.is_other_value(b"R") {
                    Ok(PDFObject::Indirect(PDFIndirect::new(
                        token.to_u32()?,
                        token_2.to_u16()?,
                    )))
                } else {
                    self.seek_to(pos)?;
                    Ok(PDFObject::Number(PDFNumber::Real(token.to_f64()?)))
                }
            }
            Token::StartHexString => {
                let hex = self.read_hex_string()?;
                Ok(PDFObject::String(hex))
            }
            Token::StartLiteralString => {
                let s = self.read_literal_string()?;
                Ok(PDFObject::String(s))
            }
            Token::StartArray => {
                let mut objs = Vec::new();
                while !self.check_next_token(&Token::EndArray)? {
                    let obj = self.read_object()?;
                    objs.push(obj);
                }
                Ok(PDFObject::Arrray(objs))
            }
            Token::StartDict => {
                let mut dict = HashMap::new();
                while !self.check_next_token(&Token::EndDict)? {
                    let keyword = self.next_token()?;
                    let obj = self.read_object()?;
                    dict.insert(keyword.to_string()?, obj);
                }
                if self.check_next_token(&Token::Other(b"stream".to_vec()))? {
                    let st = self.read_stream(dict)?;
                    return Ok(PDFObject::Stream(st));
                }
                Ok(PDFObject::Dictionary(dict))
            }
            Token::Name(_) => Ok(PDFObject::Name(PDFName::new(&token.to_string()?))),
            Token::Other(bytes) => match bytes.as_slice() {
                b"true" => Ok(PDFObject::Bool(true)),
                b"false" => Ok(PDFObject::Bool(false)),
                b"null" => Ok(PDFObject::Null),
                _ => Err(PDFError::InvalidSyntax(format!(
                    "invalid other token:{:?}",
                    bytes
                ))),
            },
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

    pub fn read_fixlen_block(&mut self, len: usize) -> PDFResult<Vec<u8>> {
        let mut buf = vec![0; len];
        self.stream.read(&mut buf).map_err(|e| PDFError::IO {
            source: e,
            msg: "read fixlen faild".to_string(),
        })?;
        Ok(buf)
    }

    fn read_stream(&mut self, dict: PDFDictionary) -> PDFResult<PDFStream> {
        self.move_next_line()?;
        let pos = self.current_position()?;
        let buf = if let Some(length) = dict.get("Length") {
            let len = length.as_u32()?;
            self.read_fixlen_block(len as usize)?
        } else {
            let end_pos = self.find_end_stream_content()?;
            let len = end_pos - pos;
            self.read_fixlen_block(len as usize)?
        };
        let mut stream = PDFStream::new(pos, dict);
        stream.set_buffer(buf);
        Ok(stream)
    }

    // search stream body end position not move current position
    fn find_end_stream_content(&mut self) -> PDFResult<u64> {
        let end_pos = self.find_tag(b"endstream")?;
        let line_marker = self.find_end_of_line_marker(end_pos - 2)?;
        let end = end_pos - line_marker;
        Ok(end)
    }

    // search a word , and return word start postion
    pub fn find_tag(&mut self, tag: &[u8]) -> PDFResult<u64> {
        let pos = self.current_position()?;
        let mut cur = 0;
        loop {
            let ch = self.read_next_char()?;
            if ch == tag[cur] {
                if cur == tag.len() - 1 {
                    break;
                } else {
                    cur += 1;
                }
            } else if ch == tag[0] {
                cur = 1;
            } else {
                cur = 0;
            }
        }
        let start_tag_pos = self.current_position()? - tag.len() as u64;
        self.seek_to(pos)?;
        Ok(start_tag_pos)
    }

    fn find_end_of_line_marker(&mut self, start: u64) -> PDFResult<u64> {
        self.seek_to(start)?;
        match self.read_end_of_line_marker()? {
            2 => Ok(2),
            _ => self.find_end_of_line_marker(start + 1),
        }
    }

    fn read_end_of_line_marker(&mut self) -> PDFResult<u64> {
        match self.read_next_char()? {
            b'\r' => match self.read_next_char()? {
                b'\n' => Ok(2),
                _ => {
                    self.step_back()?;
                    Ok(1)
                }
            },
            b'\n' => Ok(1),
            _ => {
                self.step_back()?;
                Ok(0)
            }
        }
    }

    pub fn seek_to(&mut self, pos: u64) -> PDFResult<()> {
        self.stream
            .seek(SeekFrom::Start(pos))
            .map_err(|e| PDFError::IO {
                source: e,
                msg: "seek error".to_string(),
            })?;
        Ok(())
    }

    pub fn move_next_token(&mut self) -> PDFResult<()> {
        let mut ch = self.read_next_char()?;
        loop {
            if is_whitespace(ch) {
                ch = self.read_next_char()?;
                continue;
            }
            if ch != b'%' {
                self.step_back()?;
                break;
            }
            loop {
                ch = self.read_next_char()?;
                if is_end_of_line(ch) {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn step_back(&mut self) -> PDFResult<()> {
        self.stream
            .seek(SeekFrom::Current(-1))
            .map_err(|e| PDFError::IO {
                source: e,
                msg: "Faild step back".to_string(),
            })?;
        Ok(())
    }

    pub fn check_next_token(&mut self, expected: &Token) -> PDFResult<bool> {
        let pos = self.current_position()?;
        let token = self.next_token()?;
        if &token == expected {
            return Ok(true);
        }
        self.seek_to(pos)?;
        Ok(false)
    }

    pub fn next_token(&mut self) -> PDFResult<Token> {
        self.move_next_token()?;
        if !self.is_eof()? {
            return Ok(Token::Eof);
        }
        let mut ch = self.read_next_char()?;
        let mut bytes = Vec::new();
        if is_delimiter(ch) {
            match ch {
                b'/' => {
                    ch = self.read_next_char()?;
                    while is_number(ch) || is_regular(ch) {
                        bytes.push(ch);
                        ch = self.read_next_char()?;
                    }
                    self.step_back()?;
                    return Ok(Token::Name(bytes));
                }
                b'<' => {
                    ch = self.read_next_char()?;
                    match ch {
                        b'<' => {
                            return Ok(Token::StartDict);
                        }
                        _ => {
                            self.step_back()?;
                            return Ok(Token::StartHexString);
                        }
                    }
                }
                b'>' => {
                    ch = self.read_next_char()?;
                    match ch {
                        b'>' => {
                            return Ok(Token::EndDict);
                        }
                        _ => {
                            self.step_back()?;
                            return Ok(Token::EndHexString);
                        }
                    }
                }
                b'(' => {
                    return Ok(Token::StartLiteralString);
                }
                b')' => {
                    return Ok(Token::EndLiteralString);
                }
                b'[' => {
                    return Ok(Token::StartArray);
                }
                b']' => {
                    return Ok(Token::EndArray);
                }
                _ => {
                    bytes.push(ch);
                    return Ok(Token::Other(bytes));
                }
            }
        }
        let mut number = is_number(ch);
        bytes.push(ch);
        loop {
            ch = self.read_next_char()?;
            if is_whitespace(ch) || is_delimiter(ch) {
                break;
            }
            if !is_number(ch) {
                number = false;
            }
            bytes.push(ch);
        }
        self.step_back()?;
        if number {
            Ok(Token::Number(bytes))
        } else {
            Ok(Token::Other(bytes))
        }
    }

    fn read_next_char(&mut self) -> PDFResult<u8> {
        if !self.is_eof()? {
            return Err(PDFError::Eof {
                msg: "reach eof".to_string(),
            });
        }
        let mut buf: [u8; 1] = [0; 1];
        let _ = self.stream.read(&mut buf[..]).map_err(|e| PDFError::IO {
            source: e,
            msg: "Faild peek byte".to_string(),
        })?;
        Ok(buf[0])
    }

    fn is_eof(&mut self) -> PDFResult<bool> {
        let pos = self.current_position()?;
        Ok(pos < self.size)
    }

    pub fn current_position(&mut self) -> PDFResult<u64> {
        self.stream.stream_position().map_err(|e| PDFError::IO {
            source: e,
            msg: "faild to get position".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn new_parser(content: &str) -> SyntaxParser<Cursor<&[u8]>> {
        let cursor = Cursor::new(content.as_bytes());
        SyntaxParser::try_new(cursor).unwrap()
    }

    #[test]
    fn test_read_real() {
        let content = "-80 ";
        let mut parser = new_parser(content);
        let token = parser.next_token().unwrap();
        assert!(token.is_number());
    }

    #[test]
    fn test_move_next_line() {
        let content = "abc\r\ndef ";
        let mut parser = new_parser(content);
        assert_eq!(parser.size, 9);
        assert_eq!(parser.current_position().unwrap(), 0);
        parser.move_next_line().unwrap();
        let word = parser.next_token().unwrap();
        assert_eq!(word, Token::Other(vec![b'd', b'e', b'f']));
    }

    #[test]
    fn test_read_literal_string() {
        let content = r#"These two strings are the same.) "#;
        let content_b = r#"These \
two strings \
are the same.) "#;
        let mut parser = new_parser(content);
        let string = parser.read_literal_string().unwrap();
        let result = string.to_string();

        let mut parser = new_parser(content_b);
        let string_b = parser.read_literal_string().unwrap();
        let result_b = string_b.to_string();
        assert_eq!(result, result_b);

        let content_c = r#"\012) "#;
        let mut parser = new_parser(content_c);
        let string_c = parser.read_literal_string().unwrap();
        let result_c = string_c.to_string();
        assert_eq!(result_c, "\n");
    }
    #[test]
    fn test_read_hex_string() {
        let content = r#"0050> "#;
        let mut parser = new_parser(content);
        let string = parser.read_hex_string().unwrap();
        assert_eq!([48, 48, 53, 48], string.bytes());
    }

    #[test]
    fn test_read_name_word() {
        let content = "/Name ";
        let mut parser = new_parser(content);
        let word = parser.next_token().unwrap();
        println!("{:?}", word);
    }

    #[test]
    fn test_parse_dict() {
        let content = r#"<< /Type /Page
           /Parent 4 0 R
           /MediaBox [ 0 0 612 792 ]
           /Resources << /Font << /F3 7 0 R
           /F5 9 0 R
           /F7 11 0 R
           >>
           /ProcSet [ /PDF ]
           >>
           /Contents 12 0 R
           /Thumb 14 0 R
           /Annots [ 23 0 R
           24 0 R
           ]
           >> endobj "#;
        let mut parser = new_parser(content);
        let dict = parser.read_object().unwrap();
        println!("{:?}", dict);
    }
}
