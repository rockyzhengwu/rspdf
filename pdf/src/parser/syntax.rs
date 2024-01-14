use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

use crate::errors::{PDFError, PDFResult};
use crate::lexer::{buf_to_number, buf_to_real};
use crate::object::{PDFArray, PDFName, PDFObject, PDFStream};
use crate::object::{PDFIndirect, PDFNumber, PDFString};
use crate::parser::character_set::{
    hex_to_u8, is_delimiter, is_end_of_line, is_number, is_regular, is_whitespace, is_xdigit,
};

enum StringStatus {
    Normal,
    Backslash,
    Octal,
    FinishOctal,
    CarriageReturn,
}

pub struct SyntaxParser<T: Seek + Read> {
    stream: T,
    size: u64,
}

#[derive(Debug, Default)]
pub struct Word {
    bytes: Vec<u8>,
    is_number: bool,
}

impl Word {
    pub fn value_to_string(&self) -> PDFResult<String> {
        if self.bytes.is_empty() {
            return Ok("".to_string());
        }
        let mut s = String::from_utf8(self.bytes.clone()).map_err(|e| {
            PDFError::InvalidSyntax(format!("faild convert name to string:{:?}", e))
        })?;
        s.remove(0);
        Ok(s)
    }
}

impl<T: Seek + Read> SyntaxParser<T> {
    pub fn new(mut stream: T) -> PDFResult<Self> {
        let size = stream.seek(SeekFrom::End(0)).map_err(|e| {
            PDFError::InvalidSyntax(format!("input stream not seek failed :{:?}", e))
        })?;
        stream
            .rewind()
            .map_err(|e| PDFError::InvalidSyntax(format!("input stream failed rewind:{:?}", e)))?;

        Ok(SyntaxParser { stream, size })
    }

    fn move_next_line(&mut self) -> PDFResult<()> {
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
    fn read_hex_string(&mut self) -> PDFResult<PDFString> {
        let mut ch = self.read_next_char()?;
        let mut code: u8 = 0;
        let mut bytes = Vec::new();
        let mut first = true;
        loop {
            if is_xdigit(ch) {
                if ch == b'>' {
                    break;
                }
                let val = hex_to_u8(ch);
                if first {
                    code = val * 16;
                } else {
                    code += val;
                    bytes.push(code);
                }
                first = !first
            }
            ch = self.read_next_char()?;
        }
        if first {
            bytes.push(code);
        }
        Ok(PDFString::HexString(bytes))
    }

    fn read_literal_string(&mut self) -> PDFResult<PDFString> {
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
                        0..=7 => {
                            status = StringStatus::Octal;
                            esc_octal = ch
                        }
                        b'\r' => status = StringStatus::CarriageReturn,
                        b'n' => bytes.push(b'\n'),
                        b'r' => bytes.push(b'\r'),
                        b't' => bytes.push(b'\t'),
                        b'b' => bytes.push(8),
                        b'f' => bytes.push(12),
                        b'\n' => {} //donothing
                        _ => bytes.push(ch),
                    }
                    status = StringStatus::Normal;
                }
                StringStatus::Octal => match ch {
                    0..=7 => {
                        esc_octal = esc_octal * 8 + ch;
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
                        0..=7 => {
                            esc_octal = esc_octal * 8 + ch;
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

    fn read_object(&mut self) -> PDFResult<PDFObject> {
        let word = self.next_word()?;
        if word.is_number {
            let pos = self.current_position()?;
            let word_2 = self.next_word()?;
            if !word_2.is_number {
                self.seek_to(pos)?;
                return Ok(PDFObject::Number(PDFNumber::Real(buf_to_real(&word.bytes))));
            }
            let word_3 = self.next_word()?;
            if word_3.bytes != b"R" {
                self.seek_to(pos)?;
                return Ok(PDFObject::Number(PDFNumber::Real(buf_to_real(&word.bytes))));
            }
            return Ok(PDFObject::Indirect(PDFIndirect::new(
                buf_to_number(&word.bytes),
                buf_to_number(&word_2.bytes),
            )));
        }
        match word.bytes.as_slice() {
            b"true" => Ok(PDFObject::Bool(true)),
            b"false" => Ok(PDFObject::Bool(false)),
            b"<" => {
                let hex = self.read_hex_string()?;
                Ok(PDFObject::String(hex))
            }
            b"(" => {
                let s = self.read_literal_string()?;
                Ok(PDFObject::String(s))
            }
            b"[" => {
                let mut objs = Vec::new();
                while !self.check_next_word(b"]")? {
                    let obj = self.read_object()?;
                    objs.push(obj);
                }
                Ok(PDFObject::Arrray(objs))
            }
            b"<<" => {
                let mut dict = HashMap::new();
                while !self.check_next_word(b">>")? {
                    let keyword = self.next_word()?;
                    let obj = self.read_object()?;
                    dict.insert(PDFName::new(&keyword.value_to_string()?), obj);
                }
                if self.check_next_word(b"stream")? {
                    let st = self.read_stream()?;
                    return Ok(PDFObject::Stream(st));
                }
                Ok(PDFObject::Dictionary(dict))
            }
            _ => {
                if let Some(fc) = word.bytes.first() {
                    if fc == &b'/' {
                        let mut s = String::from_utf8(word.bytes).map_err(|e| {
                            PDFError::InvalidSyntax(format!("faild parse PDFname object:{:?}", e))
                        })?;
                        s.remove(0);
                        return Ok(PDFObject::Name(PDFName::new(&s)));
                    }
                }

                Err(PDFError::InvalidSyntax(format!("invalid word:{:?}", word)))
            }
        }
    }

    fn read_stream(&mut self) -> PDFResult<PDFStream> {
        unimplemented!();
    }

    fn seek_to(&mut self, pos: u64) -> PDFResult<()> {
        self.stream
            .seek(SeekFrom::Start(pos))
            .map_err(|e| PDFError::IO {
                source: e,
                msg: "seek error".to_string(),
            })?;
        Ok(())
    }

    fn move_next_word(&mut self) -> PDFResult<()> {
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

    fn check_next_word(&mut self, value: &[u8]) -> PDFResult<bool> {
        let pos = self.current_position()?;
        let word = self.next_word()?;
        if word.bytes == value {
            return Ok(true);
        }
        self.seek_to(pos)?;
        Ok(false)
    }

    fn next_word(&mut self) -> PDFResult<Word> {
        self.move_next_word()?;
        let mut ch = self.read_next_char()?;
        let mut word = Word {
            bytes: Vec::new(),
            is_number: false,
        };
        word.bytes.push(ch);
        if is_delimiter(ch) {
            match ch {
                b'/' => {
                    ch = self.read_next_char()?;
                    while is_number(ch) || is_regular(ch) {
                        word.bytes.push(ch);
                        ch = self.read_next_char()?;
                    }
                    self.step_back()?;
                    return Ok(word);
                }
                b'<' => {
                    ch = self.read_next_char()?;
                    if ch == b'<' {
                        word.bytes.push(ch);
                    } else {
                        self.step_back()?;
                    }
                    return Ok(word);
                }
                b'>' => {
                    ch = self.read_next_char()?;
                    if ch == b'>' {
                        word.bytes.push(ch);
                    } else {
                        self.step_back()?;
                    }
                    return Ok(word);
                }
                _ => {
                    return Ok(word);
                }
            }
        }
        word.is_number = is_number(ch);
        loop {
            ch = self.read_next_char()?;
            if is_whitespace(ch) || is_delimiter(ch) {
                break;
            }
            if !is_number(ch) {
                word.is_number = false;
            }
            word.bytes.push(ch);
        }
        self.step_back()?;
        Ok(word)
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

    fn check_next_char(&mut self, c: u8) -> PDFResult<bool> {
        let pos = self.current_position()?;
        let ch = self.read_next_char()?;
        if ch == c {
            return Ok(true);
        }
        self.seek_to(pos)?;
        Ok(false)
    }

    fn is_eof(&mut self) -> PDFResult<bool> {
        let pos = self.current_position()?;
        Ok(pos < self.size)
    }

    fn current_position(&mut self) -> PDFResult<u64> {
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
        SyntaxParser::new(cursor).unwrap()
    }

    #[test]
    fn test_read_real() {
        let content = "-80 ";
        let mut parser = new_parser(content);
        let word = parser.next_word().unwrap();
        assert!(word.is_number);
    }

    #[test]
    fn test_move_next_line() {
        let content = "abc\r\ndef ";
        let mut parser = new_parser(content);
        assert_eq!(parser.size, 9);
        assert_eq!(parser.current_position().unwrap(), 0);
        parser.move_next_line().unwrap();
        let word = parser.next_word().unwrap();
        assert_eq!(word.bytes, [b'd', b'e', b'f']);
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
    }
    #[test]
    fn test_read_name_word() {
        let content = "/Name ";
        let mut parser = new_parser(content);
        let word = parser.next_word().unwrap();
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
           >> end "#;
        let mut parser = new_parser(content);
        let _dict = parser.read_object().unwrap();
    }
}
