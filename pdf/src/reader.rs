use std::cell::RefCell;
use std::char;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::character::{self, is_white_space, u16_from_buffer, u32_from_buffer};
use crate::error::PdfError;
use crate::error::Result;
use crate::object::array::PdfArray;
use crate::object::bool::PdfBool;
use crate::object::dictionary::PdfDict;
use crate::object::name::PdfName;
use crate::object::number::PdfNumber;
use crate::object::stream::PdfStream;
use crate::object::string::{PdfHexString, PdfLiteral};
use crate::object::PdfObject;

#[derive(Debug)]
pub struct PdfReader {
    buffer: Vec<u8>,
    position: RefCell<usize>,
}

#[derive(Debug)]
pub enum Token<'a> {
    Other(&'a [u8]),
    Number(&'a [u8], bool),
    StartName,
    StartHexString,
    EndHexString,
    StartLiteralString,
    EndLiteralString,
    StartArray,
    EndArray,
    StartDict,
    EndDict,
    StartComment,
    Eof,
}

enum StringStatus {
    Normal,
    Backslash,
    Octal,
    FinishOctal,
    CarriageReturn,
}

impl<'a> Token<'a> {
    pub fn is_other_key(&self, key: &[u8]) -> bool {
        match self {
            &Token::Other(v) => v == key,
            _ => false,
        }
    }

    pub fn buffer(&self) -> Option<&'a [u8]> {
        match self {
            Token::Other(buf) => Some(buf),
            Token::Number(buf, _) => Some(buf),
            _ => None,
        }
    }
}

impl PdfReader {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            position: RefCell::new(0),
        }
    }

    pub fn peek_token(&self) -> Result<Token> {
        let pos = self.position.clone();
        let token = self.read_token()?;
        *self.position.borrow_mut() = *pos.borrow();
        Ok(token)
    }

    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    pub fn new_from_file<P: AsRef<Path>>(p: P) -> Result<Self> {
        let mut f = std::fs::File::open(p).map_err(|e| {
            PdfError::Reader(format!("Create PdfReader Failed to read from {:?}", e))
        })?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)
            .map_err(|e| PdfError::Reader(format!("Crate PdfReader Failed read data {:?}", e)))?;
        Ok(Self::new(data))
    }

    pub fn read_bytes(&self, n: usize) -> Result<&[u8]> {
        let mut pos = self.position.borrow_mut();
        if *pos >= self.buffer.len() {
            return Err(PdfError::File("PdfReader read_bytes reach Eof".to_string()));
        }
        let end = (*pos + n).min(self.buffer.len());
        let data = &self.buffer[*pos..end];
        *pos = end;
        Ok(data)
    }

    pub fn reset_offset(&self, offset: usize) {
        let mut pos = self.position.borrow_mut();
        if offset > self.buffer.len() {
            *pos = self.buffer.len();
        } else {
            *pos = offset
        }
    }

    pub fn read_byte(&self) -> Result<&u8> {
        let mut pos = self.position.borrow_mut();
        if *pos >= self.buffer.len() {
            return Err(PdfError::Reader(
                "PdfReader read_bytes reach Eof".to_string(),
            ));
        }
        let v = &self.buffer[*pos];
        *pos += 1;
        Ok(v)
    }

    pub fn peek_byte(&self) -> Result<&u8> {
        if self.is_eof() {
            return Err(PdfError::File("Pdf reader peek_byte reach Eof".to_string()));
        }
        let pos = self.position.borrow();
        let v = &self.buffer[*pos];
        Ok(v)
    }

    pub fn peek_bytes(&self, len: usize) -> Result<&[u8]> {
        let pos = self.position.borrow();
        if *pos + len > self.buffer.len() {
            return Err(PdfError::Reader(
                "Pdf Reader peek_bytes reach Eof".to_string(),
            ));
        }
        Ok(&self.buffer[*pos..*pos + len])
    }

    pub fn read_line(&self) -> Result<&[u8]> {
        let start = self.position.clone();
        let mut strip = 1;
        while let Ok(ch) = self.read_byte() {
            if ch == &character::LINE_FEED {
                break;
            } else if ch == &character::CARRIAGE_RETURN {
                let nch = self.peek_byte()?;
                if nch == &character::LINE_FEED {
                    strip = 2;
                    self.read_byte()?;
                    break;
                }
            }
        }
        let data = &self.buffer[*start.borrow()..*self.position.borrow() - strip];
        Ok(data)
    }

    pub fn read_token(&self) -> Result<Token> {
        self.skip_white_space()?;
        let start = self.position.clone();
        let ch = self.read_byte()?;
        if character::is_number(ch) {
            let mut is_real = false;
            if ch == &b'.' {
                is_real = true;
            }
            while let Ok(nch) = self.peek_byte() {
                if nch == &b'.' {
                    is_real = true;
                }
                if !character::is_number(nch) {
                    break;
                }
                self.read_byte()?;
            }
            let data = &self.buffer[*start.borrow()..*self.position.borrow()];
            return Ok(Token::Number(data, is_real));
        }
        match *ch {
            character::SOLIDUS => Ok(Token::StartName),
            character::LEFT_PARENTHESIS => Ok(Token::StartLiteralString),
            character::LESS_THAN_SIGN => {
                let nch = self.peek_byte()?;
                if nch == &character::LESS_THAN_SIGN {
                    self.read_byte()?;
                    Ok(Token::StartDict)
                } else {
                    Ok(Token::StartHexString)
                }
            }
            character::LEFT_SQUARE_BRACKET => Ok(Token::StartArray),
            character::PERCENT_SIGN => Ok(Token::StartComment),
            _ => {
                while let Ok(nch) = self.peek_byte() {
                    if character::is_white_space(nch) || character::is_delimiter(nch) {
                        break;
                    }
                    self.read_byte()?;
                }
                let data = &self.buffer[*start.borrow()..*self.position.borrow()];
                Ok(Token::Other(data))
            }
        }
    }

    pub fn skip_white_space(&self) -> Result<()> {
        while let Ok(ch) = self.peek_byte() {
            if character::is_white_space(ch) {
                self.read_byte()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn read_indirect_object(&self, offset: usize) -> Result<PdfObject> {
        self.reset_offset(offset);
        let _n = self.read_token()?;
        let _g = self.read_token()?;
        let obj_key = self.read_token()?;
        assert!(obj_key.is_other_key(b"obj"));
        self.read_object()
    }

    pub fn read_object(&self) -> Result<PdfObject> {
        self.skip_white_space()?;
        let token = self.read_token()?;
        while !self.is_eof() {
            match token {
                Token::Eof => {
                    return Err(PdfError::Reader("Read object reach Eof".to_string()));
                }
                Token::StartComment => {
                    self.read_comment()?;
                }

                Token::StartHexString => {
                    let s = self.read_hex_string()?;
                    return Ok(PdfObject::HexString(s));
                }
                Token::StartLiteralString => {
                    let s = self.read_literial_string()?;
                    return Ok(PdfObject::LiteralString(s));
                }
                Token::StartArray => {
                    let arr = self.read_array()?;
                    return Ok(PdfObject::Array(arr));
                }
                Token::StartDict => {
                    let dict = self.read_dict()?;
                    return Ok(PdfObject::Dict(dict));
                }
                Token::Number(buf, is_real) => {
                    let pos = self.position.clone();
                    if let Token::Number(n2, _) = self.read_token()? {
                        let r = self.read_token()?;
                        if r.is_other_key(b"R") {
                            let object_id = u32_from_buffer(buf).map_err(|e| {
                                PdfError::Reader(format!("Indirect ObjectId read error:{:?}", e))
                            })?;
                            let gen = u16_from_buffer(n2).map_err(|e| {
                                PdfError::Reader(format!("Indirect Object gen error:{:?}", e))
                            })?;
                            return Ok(PdfObject::Indirect((object_id, gen)));
                        }
                    }
                    self.reset_offset(*pos.borrow());
                    let num = PdfNumber::from_buffer(buf, is_real);
                    return Ok(PdfObject::Number(num));
                }
                Token::StartName => {
                    let name = self.read_name()?;
                    return Ok(PdfObject::Name(name));
                }
                _ => {
                    if token.is_other_key(b"true") {
                        return Ok(PdfObject::Bool(PdfBool(true)));
                    } else if token.is_other_key(b"false") {
                        return Ok(PdfObject::Bool(PdfBool(false)));
                    }
                    if token.is_other_key(b"null") {
                        return Ok(PdfObject::Null);
                    }
                    return Err(PdfError::Reader(format!(
                        "Read Object Error unexpected token :{:?}",
                        token
                    )));
                }
            }
        }
        Err(PdfError::Reader("Read Object Error reach EOF".to_string()))
    }

    fn find_end_stream_content(&self) -> Result<usize> {
        let pos = self.current_pos();
        let end_pos = self.find_tag(b"endstream")?;
        let line_marker = self.find_end_of_line_marker(end_pos - 2)?;
        let end = end_pos - line_marker;
        self.reset_offset(pos);
        Ok(end)
    }

    fn find_end_of_line_marker(&self, start: usize) -> Result<usize> {
        self.reset_offset(start);
        self.read_end_of_line_marker()
    }

    fn read_end_of_line_marker(&self) -> Result<usize> {
        match self.read_byte()? {
            b'\r' => match self.read_byte()? {
                b'\n' => Ok(2),
                _ => Ok(1),
            },
            b'\n' => Ok(1),
            _ => Ok(0),
        }
    }

    fn find_tag(&self, tag: &[u8]) -> Result<usize> {
        let pos = self.current_pos();
        let mut cur = 0;
        loop {
            let ch = self.read_byte()?;
            if ch == &tag[cur] {
                if cur == tag.len() - 1 {
                    break;
                }
                cur += 1;
            } else if ch == &tag[0] {
                cur = 1;
            } else {
                cur = 0;
            }
            if pos + cur > self.size() {
                return Err(PdfError::Reader(format!(
                    "can't find tag reach eof :{:?}",
                    tag
                )));
            }
        }
        let start_tag_pos = self.current_pos() - tag.len();
        self.reset_offset(pos);
        Ok(start_tag_pos)
    }

    pub fn read_name(&self) -> Result<PdfName> {
        self.skip_white_space()?;
        let mut bytes = Vec::new();
        while let Ok(ch) = self.peek_byte() {
            if ch == &character::NUMBER_SIGN {
                self.read_byte()?;
                let c = self.read_bytes(2)?;
                let s = String::from_utf8(c.to_owned()).unwrap();
                let c = u8::from_str_radix(s.as_str(), 16).unwrap();
                bytes.push(c);
            } else if character::is_white_space(ch) || character::is_delimiter(ch) {
                break;
            } else {
                let c = self.read_byte()?;
                bytes.push(c.to_owned());
            }
        }
        PdfName::from_buffer(bytes)
    }
    pub fn read_stream(&self) -> Result<PdfStream> {
        let dict = self.read_dict()?;
        let pos = self.position.clone();
        let next = self.read_token()?;
        if next.is_other_key(b"stream") {
            // TODO handle this length is not in Dict, read to keyward endstream

            let pos = self.current_pos();
            let c = self.peek_byte()?;
            if *c == b'\r' {
                self.read_byte()?;
                let nc = self.peek_byte()?;
                if *nc == b'\n' {
                    self.read_byte()?;
                } else {
                    self.reset_offset(pos);
                }
            } else if *c == b'\n' {
                self.read_byte()?;
            }
            if let Some(PdfObject::Number(length)) = dict.get("Length") {
                let data = self.read_bytes(length.integer() as usize)?;
                Ok(PdfStream::new(dict, data.to_owned()))
            } else {
                let end_of_stream = self.find_end_stream_content()?;
                let len = end_of_stream - self.current_pos();
                let data = self.read_bytes(len)?;
                Ok(PdfStream::new(dict, data.to_owned()))
            }
        } else {
            return Err(PdfError::Reader(
                "Stream need stream as keyword".to_string(),
            ));
        }
    }
    pub fn read_stream_data(&self, length: usize) -> Result<Vec<u8>> {
        let next = self.read_token()?;
        assert!(next.is_other_key(b"stream"));
        let pos = self.current_pos();
        let c = self.peek_byte()?;
        if *c == b'\r' {
            self.read_byte()?;
            let nc = self.peek_byte()?;
            if *nc == b'\n' {
                self.read_byte()?;
            } else {
                self.reset_offset(pos);
            }
        } else if *c == b'\n' {
            self.read_byte()?;
        }
        let data = self.read_bytes(length)?;
        let token = self.read_token()?;
        assert!(token.is_other_key(b"endstream"));
        Ok(data.to_owned())
    }

    pub fn read_array(&self) -> Result<PdfArray> {
        let mut elements = Vec::new();
        while let Ok(ch) = self.peek_byte() {
            if ch == &character::RIGHT_SQUARE_BRACKET {
                self.read_byte()?;
                break;
            }
            let obj = self.read_object()?;
            elements.push(obj);
            self.skip_white_space()?;
        }
        Ok(PdfArray::new(elements))
    }

    pub fn read_hex_string(&self) -> Result<PdfHexString> {
        let mut ch = self.read_byte()?;
        let mut bytes = Vec::new();
        loop {
            if self.is_eof() {
                break;
            }
            if is_white_space(ch) {
                ch = self.read_byte()?;
                continue;
            }
            if ch == &character::GREATER_THAN_SIGN {
                break;
            }
            bytes.push(ch.to_owned());
            ch = self.read_byte()?
        }
        if bytes.len() % 2 == 1 {
            bytes.push(b'0');
        }
        Ok(PdfHexString::new(bytes))
    }

    pub fn read_literial_string(&self) -> Result<PdfLiteral> {
        let mut nest_level: i32 = 0;
        let mut status: StringStatus = StringStatus::Normal;
        let mut ch = self.read_byte()?;
        let mut bytes = Vec::new();
        let mut esc_octal = String::new();
        loop {
            match status {
                StringStatus::Normal => match *ch {
                    character::LEFT_PARENTHESIS => {
                        bytes.push(ch.to_owned());
                        nest_level += 1;
                    }
                    character::RIGHT_PARENTHESIS => {
                        if nest_level == 0 {
                            return Ok(PdfLiteral::new(bytes));
                        }
                        bytes.push(ch.to_owned());
                        nest_level -= 1;
                    }
                    character::REVERSE_SOLIDUS => {
                        status = StringStatus::Backslash;
                    }
                    _ => {
                        bytes.push(ch.to_owned());
                    }
                },
                StringStatus::Backslash => {
                    match ch {
                        b'0'..=b'7' => {
                            status = StringStatus::Octal;
                            esc_octal.push(ch.to_owned() as char);
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
                            bytes.push(ch.to_owned())
                        }
                    }
                }
                StringStatus::Octal => match ch {
                    b'0'..=b'7' => {
                        esc_octal.push(ch.to_owned() as char);
                        status = StringStatus::FinishOctal;
                    }
                    _ => {
                        let v = u8::from_str_radix(esc_octal.as_str(), 8).unwrap();
                        bytes.push(v);
                        esc_octal.clear();
                        status = StringStatus::Normal;
                    }
                },
                StringStatus::FinishOctal => {
                    status = StringStatus::Normal;
                    match ch {
                        b'0'..=b'7' => {
                            esc_octal.push(ch.to_owned() as char);
                            let v = u8::from_str_radix(esc_octal.as_str(), 8).unwrap();
                            esc_octal.clear();
                            bytes.push(v);
                        }
                        _ => {
                            let v = u8::from_str_radix(esc_octal.as_str(), 8).unwrap();
                            esc_octal.clear();
                            bytes.push(v);
                        }
                    }
                }
                StringStatus::CarriageReturn => {
                    status = StringStatus::Normal;
                    if ch != &character::LINE_FEED {
                        continue;
                    }
                }
            }
            if self.is_eof() {
                return Ok(PdfLiteral::new(bytes));
            }
            ch = self.read_byte()?;
        }
    }

    pub fn read_comment(&self) -> Result<()> {
        self.read_line()?;
        Ok(())
    }

    pub fn is_eof(&self) -> bool {
        *self.position.borrow() >= self.buffer.len()
    }

    pub fn read_number(&self) -> Result<PdfNumber> {
        self.skip_white_space()?;
        let pos = self.position.clone();
        let mut is_real = false;
        while let Ok(ch) = self.peek_byte() {
            if ch == &b'.' {
                is_real = true;
            }
            if !character::is_number(ch) {
                break;
            }
            self.read_byte()?;
        }
        let buffer = &self.buffer[*pos.borrow()..*self.position.borrow()];
        Ok(PdfNumber::from_buffer(buffer, is_real))
    }

    pub fn current_pos(&self) -> usize {
        *self.position.borrow()
    }

    pub fn read_dict(&self) -> Result<PdfDict> {
        self.skip_white_space()?;
        let mut values = HashMap::new();
        while self.peek_bytes(2)? != b">>" {
            let start_name = self.read_byte()?;
            assert_eq!(start_name, &b'/');
            let name = self.read_name()?;
            let obj = self.read_object()?;
            values.insert(name.to_string(), obj);
            self.skip_white_space()?;
        }
        let end_dict = self.read_bytes(2)?;
        assert_eq!(end_dict, b">>");
        Ok(PdfDict::new(values))
    }

    pub fn read_until_reach(&self, tag: &[u8]) -> Result<Vec<u8>> {
        let mut cur = 0;
        let mut res = Vec::new();
        loop {
            let ch = self.read_byte()?.to_owned();
            res.push(ch);
            if ch == tag[cur] {
                if cur == tag.len() - 1 {
                    break;
                }
                cur += 1;
            } else if ch == tag[0] {
                cur = 1;
            } else {
                cur = 0;
            }
        }
        for _ in 0..tag.len() {
            res.pop();
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::object::array::PdfArray;
    use crate::object::dictionary::PdfDict;
    use crate::object::name::PdfName;
    use crate::object::string::PdfLiteral;
    use crate::object::PdfObject;
    use crate::object::*;
    use crate::reader::PdfReader;

    fn new_reader(buffer: &str) -> PdfReader {
        PdfReader::new(buffer.as_bytes().to_vec())
    }

    fn parse_name(buffer: &str, expected: &str) {
        let reader = new_reader(buffer);
        let name = reader.read_name().unwrap();
        assert_eq!(name.to_string(), expected);
    }

    fn parse_string(buffer: &str, expected: &[u8]) {
        let reader = new_reader(buffer);
        let result = reader.read_literial_string().unwrap();
        let s: String = result
            .bytes()
            .iter()
            .map(|v| v.to_owned() as char)
            .collect();
        assert_eq!(s.as_bytes(), expected);
    }

    fn parse_hex(buffer: &str, expected: &[u8]) {
        let reader = new_reader(buffer);
        let result = reader.read_hex_string().unwrap();
        assert_eq!(result.raw_bytes().unwrap(), expected);
    }

    fn parse_array(buffer: &str, expected: &PdfArray) {
        let reader = new_reader(buffer);
        let result = reader.read_array().unwrap();
        assert_eq!(&result, expected);
    }

    fn parse_dict(buffer: &str, exptected: &dictionary::PdfDict) {
        let reader = new_reader(buffer);
        let result = reader.read_dict().unwrap();
        assert_eq!(&result, exptected)
    }

    #[test]
    fn test_parse_name() {
        parse_name("Name1", "Name1");
        parse_name("lime#20Green", "lime Green");
        parse_name("paired#28#29parentheses", "paired()parentheses");
        parse_name(
            "A;Name_With-Various***Characters?",
            "A;Name_With-Various***Characters?",
        );
        parse_name("1.2", "1.2");
        parse_name("$$", "$$");
        parse_name("@pattern", "@pattern");
        parse_name("The_Key_of_F#23_Minor", "The_Key_of_F#_Minor");
        parse_name("A#42", "AB");
    }
    #[test]
    fn test_parse_literal() {
        parse_string(
            r#"These \
two strings \
are the same .)"#,
            "These two strings are the same .".as_bytes(),
        );

        parse_string(
            r#"This string has an end-of-line at the end of it .
)"#,
            "This string has an end-of-line at the end of it .\n".as_bytes(),
        );

        parse_string(
            r#"This string contains \245two octal characters\307 .)"#,
            "This string contains ¥two octal charactersÇ .".as_bytes(),
        );

        parse_string(
            r#"Strings may contain balanced parentheses() and
special characters(*!&}^% and so on).)"#,
            "Strings may contain balanced parentheses() and
special characters(*!&}^% and so on)."
                .as_bytes(),
        );
        parse_string(r#")"#, "".as_bytes());
        parse_string(r#"\0053)"#, "\u{5}3".as_bytes());
        parse_string(r#"\53)"#, "+".as_bytes());
    }

    #[test]
    fn test_parse_hex() {
        parse_hex("901FA3>", &[144, 31, 163]);
        parse_hex("901FA>", &[144, 31, 160]);
    }

    #[test]
    fn test_parse_array() {
        let s = r#"549 3.144 false (Ralph) /SomeName] abc"#;
        let objs = vec![
            PdfObject::Number(number::PdfNumber::Integer(549)),
            PdfObject::Number(number::PdfNumber::Real(3.144_f32)),
            PdfObject::Bool(bool::PdfBool(false)),
            PdfObject::LiteralString(string::PdfLiteral::new("Ralph".as_bytes().to_vec())),
            PdfObject::Name(name::PdfName::new("SomeName".to_string())),
        ];
        parse_array(s, &PdfArray::new(objs));
    }
    #[test]
    fn test_parse_dict() {
        let s = r#"/Type /Example
/Subtype /DictionaryExample
/Version 0.01
/IntegerItem 12
/StringItem (a string)
/Subdictionary << /Item1 0.4
/Item2 true
/LastItem (not!)
/VeryLastItem (OK)
>>
>>"#;
        let mut expected: HashMap<String, PdfObject> = HashMap::new();
        expected.insert(
            "Type".to_string(),
            PdfObject::Name(PdfName::new("Example".to_string())),
        );
        expected.insert(
            "Subtype".to_string(),
            PdfObject::Name(PdfName::new("DictionaryExample".to_string())),
        );
        expected.insert(
            "Version".to_string(),
            PdfObject::Number(number::PdfNumber::Real(0.01)),
        );
        expected.insert(
            "IntegerItem".to_string(),
            PdfObject::Number(number::PdfNumber::Integer(12)),
        );
        expected.insert(
            "StringItem".to_string(),
            PdfObject::LiteralString(PdfLiteral::new("a string".as_bytes().to_vec())),
        );
        let mut subdict = HashMap::new();
        subdict.insert(
            "Item1".to_string(),
            PdfObject::Number(number::PdfNumber::Real(0.4)),
        );
        subdict.insert("Item2".to_string(), PdfObject::Bool(bool::PdfBool(true)));
        subdict.insert(
            "LastItem".to_string(),
            PdfObject::LiteralString(PdfLiteral::new("not!".as_bytes().to_vec())),
        );
        subdict.insert(
            "VeryLastItem".to_string(),
            PdfObject::LiteralString(PdfLiteral::new("OK".as_bytes().to_vec())),
        );
        expected.insert(
            "Subdictionary".to_string(),
            PdfObject::Dict(PdfDict::new(subdict)),
        );
        parse_dict(s, &PdfDict::new(expected));
    }

    #[test]
    fn test_read_token() {
        let buffer = "abc def";
        let reader = new_reader(buffer);
        let t = reader.read_token().unwrap();
        assert_eq!(reader.current_pos(), 3);
        assert!(t.is_other_key(b"abc"));
    }

    #[test]
    fn test_read_number() {
        let buffer = "123 456";
        let reader = new_reader(buffer);
        let t = reader.read_number().unwrap();
        assert_eq!(reader.current_pos(), 3);
        assert_eq!(t.integer(), 123);
    }
    #[test]
    fn test_read_bytes() {
        let buffer = "abc def";
        let reader = new_reader(buffer);
        let t = reader.read_bytes(3).unwrap();
        assert_eq!(t, b"abc");
        assert_eq!(reader.current_pos(), 3);
    }

    #[test]
    fn test_read_byte() {
        let buffer = "abc def";
        let reader = new_reader(buffer);
        let t = reader.read_byte().unwrap();
        assert_eq!(t, &b'a');
        assert_eq!(reader.current_pos(), 1);
    }
}
