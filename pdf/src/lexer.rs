use std::io::{Read, Seek, SeekFrom};

use crate::errors::{PDFError, PDFResult};
use crate::token::Token;

pub struct Tokenizer<T: Read + Seek> {
    input: T,
}

#[derive(PartialEq, Debug, Eq)]
enum Bytes {
    Byte(u8),
    Eof,
}

const ARRAY_OPEN: Bytes = Bytes::Byte(b'[');
const ARRAY_CLOSE: Bytes = Bytes::Byte(b']');
const LESS_THAN: Bytes = Bytes::Byte(b'<');
const GREATER_THAN: Bytes = Bytes::Byte(b'>');
const LEFT_PARENTLEHEISE: Bytes = Bytes::Byte(b'(');
const RIGHT_PARENTLEHEISE: Bytes = Bytes::Byte(b')');
const _LEFT_SQUARE_BRACKET: Bytes = Bytes::Byte(b'[');
const _RIGHT_SQUARE_BRACKET: Bytes = Bytes::Byte(b']');
const SOLIDUS: Bytes = Bytes::Byte(b'/');
const _PERSENT: Bytes = Bytes::Byte(b'%');

impl Bytes {
    fn is_white(&self) -> bool {
        match self {
            Bytes::Byte(c) => is_white(c),
            Bytes::Eof => false,
        }
    }

    fn is_delimiter(&self) -> bool {
        match self {
            Bytes::Eof => true,
            Bytes::Byte(c) => is_delimiter(c),
        }
    }

    fn is_eof(&self) -> bool {
        match self {
            Bytes::Byte(_) => false,
            Bytes::Eof => true,
        }
    }

    fn as_u8(&self) -> u8 {
        match self {
            Bytes::Byte(c) => c.to_owned(),
            Bytes::Eof => 0,
        }
    }

    fn is_persent(&self) -> bool {
        matches!(self, Bytes::Byte(b'%'))
    }
}

fn stream_length<T: Seek>(stream: &mut T) -> PDFResult<u64> {
    stream.seek(SeekFrom::End(0)).map_err(|e| PDFError::IO {
        source: e,
        msg: "Failed get stream length".to_string(),
    })
}

impl<T: Read + Seek> Tokenizer<T> {
    pub fn new(input: T) -> Self {
        Tokenizer { input }
    }

    pub fn offset(&mut self) -> PDFResult<u64> {
        self.input.stream_position().map_err(|e| PDFError::IO {
            source: e,
            msg: "Failed get current offset".to_string(),
        })
    }

    pub fn seek(&mut self, offset: u64) -> PDFResult<()> {
        self.input
            .seek(SeekFrom::Start(offset))
            .map_err(|e| PDFError::IO {
                source: e,
                msg: "Faild seek".to_string(),
            })?;
        Ok(())
    }

    pub fn peek_buffer(&mut self, buffer: &mut [u8]) -> PDFResult<()> {
        self.input.read_exact(buffer).map_err(|e| PDFError::IO {
            source: e,
            msg: "Faild peek_buffer".to_string(),
        })?;
        Ok(())
    }

    pub fn check_next(&mut self, expected: &Token) -> PDFResult<bool> {
        let ofs = self.offset()?;
        let next = self.next_token()?;
        self.seek(ofs)?;
        let old = std::mem::discriminant(&next);
        let new = std::mem::discriminant(expected);
        Ok(old == new)
    }

    pub fn next_token(&mut self) -> PDFResult<Token> {
        let token = self.read_token()?;

        match token {
            Token::PDFNumber(ref num) => {
                let ofs = self.offset()?;
                let gen = self.read_token()?;
                let third = self.read_token()?;
                match (gen, third) {
                    (Token::PDFNumber(gen), Token::PDFRef) => {
                        Ok(Token::PDFIndirect(num.to_owned(), gen))
                    }
                    _ => {
                        self.seek(ofs)?;
                        Ok(token)
                    }
                }
            }
            _ => Ok(token),
        }
    }

    pub fn skip_white(&mut self) -> PDFResult<u64> {
        let mut byte = self.peek_byte()?;
        while byte.is_white() && !byte.is_eof() {
            byte = self.peek_byte()?;
        }
        self.step_back()?;
        self.offset()
    }

    fn peek_byte(&mut self) -> PDFResult<Bytes> {
        let mut buf = [0; 1];
        let n = self.input.read(&mut buf[..]).map_err(|e| PDFError::IO {
            source: e,
            msg: "Faild peek byte".to_string(),
        })?;
        if n == 0 {
            Ok(Bytes::Eof)
        } else {
            Ok(Bytes::Byte(buf[0]))
        }
    }

    fn step_back(&mut self) -> PDFResult<()> {
        self.input
            .seek(SeekFrom::Current(-1))
            .map_err(|e| PDFError::IO {
                source: e,
                msg: "Faild step back".to_string(),
            })?;
        Ok(())
    }

    fn keyword(&mut self, buf: Vec<u8>) -> PDFResult<Token> {
        match buf.as_slice() {
            b"obj" => Ok(Token::PDFObj),
            b"endobj" => Ok(Token::PDFEndObj),
            b"stream" => Ok(Token::PDFStream),
            b"endstream" => Ok(Token::PDFEndStream),
            b"xref" => Ok(Token::PDFXRef),
            b"startxref" => Ok(Token::PDFStartXRef),
            b"trailer" => Ok(Token::PDFTrailer),
            b"R" => Ok(Token::PDFRef),
            b"false" => Ok(Token::PDFFalse),
            b"true" => Ok(Token::PDFTrue),
            _ => {
                let mut number = true;
                let mut real = true;
                for ch in buf.iter() {
                    if !is_digit(ch) {
                        number = false
                    }
                    if !is_real(ch) {
                        real = false
                    }
                    if !number && !real {
                        break;
                    }
                }
                if number {
                    Ok(Token::PDFNumber(buf_to_number(buf.as_slice())))
                } else if real {
                    Ok(Token::PDFReal(buf_to_real(buf.as_slice())))
                } else {
                    Ok(Token::PDFOther(buf))
                }
            }
        }
    }

    fn read_token(&mut self) -> PDFResult<Token> {
        let mut c = self.peek_byte()?;
        while c.is_white() && !c.is_eof() {
            c = self.peek_byte()?;
        }

        if c.is_persent() {
            loop {
                c = self.peek_byte()?;
                if c.as_u8() == b'\r' || c.as_u8() == b'\n' {
                    break;
                }
            }
        }

        if c.is_eof() {
            return Ok(Token::PDFEof);
        }

        while c.is_white() && !c.is_eof() {
            c = self.peek_byte()?;
        }

        match c {
            ARRAY_OPEN => Ok(Token::PDFOpenArray),
            ARRAY_CLOSE => Ok(Token::PDFCloseArray),
            LESS_THAN => {
                let mut cs = self.peek_byte()?;
                match cs {
                    LESS_THAN => Ok(Token::PDFOpenDict),
                    _ => {
                        let mut buf = Vec::new();
                        while cs != GREATER_THAN && !cs.is_eof() {
                            buf.push(cs.as_u8());
                            cs = self.peek_byte()?;
                        }
                        Ok(Token::PDFHexString(buf))
                    }
                }
            }
            GREATER_THAN => {
                let cs = self.peek_byte()?;
                if cs != GREATER_THAN {
                    return Err(PDFError::LexFailure("`>` can not exist single".to_string()));
                }
                Ok(Token::PDFCloseDict)
            }
            LEFT_PARENTLEHEISE => {
                let mut buf = Vec::new();
                c = self.peek_byte()?;
                while c != RIGHT_PARENTLEHEISE && !c.is_eof() {
                    let v = c.as_u8();
                    if v == 92 {
                        c = self.peek_byte()?;
                        buf.push(c.as_u8());
                    } else {
                        buf.push(v);
                    }
                    c = self.peek_byte()?;
                }
                Ok(Token::PDFLiteralString(buf))
            }
            SOLIDUS => {
                let mut cs = self.peek_byte()?;
                let mut buf = Vec::new();
                while !cs.is_white() && !cs.is_delimiter() && !cs.is_eof() {
                    buf.push(cs.as_u8());
                    cs = self.peek_byte()?;
                }
                self.step_back()?;
                Ok(Token::PDFName(
                    String::from_utf8_lossy(buf.as_slice()).to_string(),
                ))
            }
            _ => {
                let mut buf = Vec::new();
                while !c.is_delimiter() && !c.is_white() && !c.is_eof() {
                    buf.push(c.as_u8());
                    c = self.peek_byte()?;
                }
                if !c.is_eof() {
                    self.step_back()?;
                }
                self.keyword(buf)
            }
        }
    }

    pub fn find_start_xref(&mut self) -> PDFResult<u64> {
        let filesize = stream_length(&mut self.input)?;
        let size: i64 = if filesize > 1024 {
            -1024
        } else {
            -(filesize as i64)
        };

        self.input
            .seek(SeekFrom::End(size))
            .map_err(|e| PDFError::IO {
                source: e,
                msg: "Faild seek back from file to find startxref".to_string(),
            })?;
        let mut buffer: Vec<u8> = Vec::new();
        self.input
            .read_to_end(&mut buffer)
            .map_err(|e| PDFError::IO {
                source: e,
                msg: "Faild read from to find startxref".to_string(),
            })?;
        let n = buffer.len();
        let mut i = n - 9;
        while i > 0 {
            if &buffer[i..i + 9] == b"startxref" {
                break;
            }
            i -= 1;
        }
        if i == 0 {
            return Err(PDFError::InvalidFileStructure(
                "Keyword 'startxref' not found".to_string(),
            ));
        }
        i += 9;
        while is_white(&buffer[i]) {
            i += 1;
        }
        let mut startxref: u64 = 0;
        while is_digit(&buffer[i]) {
            let v = buffer[i] - 48;
            startxref = startxref * 10 + (v as u64);
            i += 1;
        }
        if startxref == 0 {
            return Err(PDFError::InvalidFileStructure(
                "startxref value can't be zero".to_string(),
            ));
        }
        Ok(startxref)
    }
}

pub fn is_white(ch: &u8) -> bool {
    matches!(ch, 0 | 9 | 10 | 12 | 13 | 32)
}

pub fn is_digit(ch: &u8) -> bool {
    (48..58).contains(ch)
}

pub fn is_number(ch: &u8) -> bool {
    is_digit(ch) || matches!(ch, b'+' | b'-')
}

pub fn is_real(ch: &u8) -> bool {
    is_number(ch) || ch == &b'.'
}

pub fn buf_to_number(buf: &[u8]) -> i64 {
    let mut res: i64 = 0;
    for c in buf {
        res = res * 10 + (c - b'0') as i64;
    }
    res
}

pub fn buf_to_real(buf: &[u8]) -> f64 {
    if buf.is_empty() {
        return 0_f64;
    }
    let mut i = 0;
    let flag: f64 = match buf[0] {
        43 => {
            i += 1;
            1_f64
        }
        45 => {
            i += 1;
            -1_f64
        }
        _ => 1_f64,
    };

    let mut ipart = 0_f64;
    while i < buf.len() && is_digit(&buf[i]) {
        ipart = ipart * 10_f64 + (buf[i] - b'0') as f64;
        i += 1
    }
    if i < buf.len() && buf[i] != b'.' {
        return flag * ipart;
    } else if i < buf.len() && buf[i] == b'.' {
        i += 1;
        let mut dpart = 0_f64;
        let mut n = 1_f64;
        while i < buf.len() && is_digit(&buf[i]) {
            n *= 10_f64;
            dpart = dpart * 10_f64 + (buf[i] - b'0') as f64;
            i += 1
        }
        return flag * (ipart + dpart / n);
    }

    flag * ipart
}

pub fn is_delimiter(ch: &u8) -> bool {
    matches!(
        ch,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::lexer::Tokenizer;
    use crate::token::Token;

    #[test]
    fn test_dict() {
        let content = b"<</Filter/FlateDecode/First 14/Length 166/N 3/Type/ObjStm>>";

        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let expected = vec![
            Token::PDFOpenDict,
            Token::PDFName("Filter".to_string()),
            Token::PDFName("FlateDecode".to_string()),
            Token::PDFName("First".to_string()),
            Token::PDFNumber(14),
            Token::PDFName("Length".to_string()),
            Token::PDFNumber(166),
            Token::PDFName("N".to_string()),
            Token::PDFNumber(3),
            Token::PDFName("Type".to_string()),
            Token::PDFName("ObjStm".to_string()),
            Token::PDFCloseDict,
        ];
        let mut result = Vec::new();
        while let Ok(token) = tokenizer.next_token() {
            if token == Token::PDFEof {
                break;
            }
            result.push(token.to_owned());
        }

        assert_eq!(expected, result);
    }

    #[test]
    fn test_nested_dict() {
        let content = b"<< /Type /Page
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
>>";

        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let expected = vec![
            Token::PDFOpenDict,
            Token::PDFName("Type".to_string()),
            Token::PDFName("Page".to_string()),
            Token::PDFName("Parent".to_string()),
            Token::PDFIndirect(4, 0),
            Token::PDFName("MediaBox".to_string()),
            Token::PDFOpenArray,
            Token::PDFNumber(0),
            Token::PDFNumber(0),
            Token::PDFNumber(612),
            Token::PDFNumber(792),
            Token::PDFCloseArray,
            Token::PDFName("Resources".to_string()),
            Token::PDFOpenDict,
            Token::PDFName("Font".to_string()),
            Token::PDFOpenDict,
            Token::PDFName("F3".to_string()),
            Token::PDFIndirect(7, 0),
            Token::PDFName("F5".to_string()),
            Token::PDFIndirect(9, 0),
            Token::PDFName("F7".to_string()),
            Token::PDFIndirect(11, 0),
            Token::PDFCloseDict,
            Token::PDFName("ProcSet".to_string()),
            Token::PDFOpenArray,
            Token::PDFName("PDF".to_string()),
            Token::PDFCloseArray,
            Token::PDFCloseDict,
            Token::PDFName("Contents".to_string()),
            Token::PDFIndirect(12, 0),
            Token::PDFName("Thumb".to_string()),
            Token::PDFIndirect(14, 0),
            Token::PDFName("Annots".to_string()),
            Token::PDFOpenArray,
            Token::PDFIndirect(23, 0),
            Token::PDFIndirect(24, 0),
            Token::PDFCloseArray,
            Token::PDFCloseDict,
        ];
        let mut result = Vec::new();
        while let Ok(token) = tokenizer.next_token() {
            if token == Token::PDFEof {
                break;
            }
            result.push(token.to_owned());
        }

        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_content() {
        let content = b"BT
/F13 12 Tf
288 720 Td
(ABC) Tj
ET";
        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let expected = vec![
            Token::PDFOther(b"BT".to_vec()),
            Token::PDFName("F13".to_string()),
            Token::PDFNumber(12),
            Token::PDFOther(b"Tf".to_vec()),
            Token::PDFNumber(288),
            Token::PDFNumber(720),
            Token::PDFOther(b"Td".to_vec()),
            Token::PDFLiteralString(b"ABC".to_vec()),
            Token::PDFOther(b"Tj".to_vec()),
            Token::PDFOther(b"ET".to_vec()),
        ];
        let mut res = Vec::new();
        while let Ok(token) = tokenizer.next_token() {
            if token == Token::PDFEof {
                break;
            }
            res.push(token);
        }
        assert_eq!(res, expected);
    }

    #[test]
    fn test_parse_reald() {
        let content = "-80";
        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let token = tokenizer.next_token().unwrap();
        assert_eq!(token, Token::PDFReal(-80.0));
    }
    #[test]
    fn test_empty() {
        let content = "";
        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let token = tokenizer.next_token().unwrap();
        assert_eq!(token, Token::PDFEof);
    }

    #[test]
    fn test_comment() {
        let content = b"abc% comment ( /% ) blah blah blah
            123";
        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let expected = vec![Token::PDFOther(b"abc".to_vec()), Token::PDFNumber(123)];
        let mut res = Vec::new();
        while let Ok(token) = tokenizer.next_token() {
            if token == Token::PDFEof {
                break;
            }
            res.push(token);
        }
        assert_eq!(res, expected);
    }
    #[test]
    fn test_escape() {
        //b"8.88 0 TD 0 Tc (O\\) Tj";
        let content = vec![
            56, 46, 56, 56, 32, 48, 32, 84, 68, 32, 48, 32, 84, 99, 32, 40, 79, 92, 92, 41, 32, 84,
            106,
        ];
        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let mut res = Vec::new();
        let expected = vec![
            Token::PDFReal(8.88),
            Token::PDFNumber(0),
            Token::PDFOther(vec![84, 68]),
            Token::PDFNumber(0),
            Token::PDFOther(vec![84, 99]),
            Token::PDFLiteralString(vec![79, 92]),
            Token::PDFOther(vec![84, 106]),
        ];

        while let Ok(token) = tokenizer.read_token() {
            if token == Token::PDFEof {
                break;
            }
            res.push(token);
        }
        assert_eq!(expected, res);
    }

    #[test]
    fn test_escape_more() {
        let content = b"(\\(\\)) Tj";
        let cursor = Cursor::new(content);
        let mut tokenizer = Tokenizer::new(cursor);
        let mut res = Vec::new();
        while let Ok(token) = tokenizer.next_token() {
            if token == Token::PDFEof {
                break;
            }
            res.push(token);
        }
        let expected = [
            Token::PDFLiteralString(vec![40, 41]),
            Token::PDFOther(vec![84, 106]),
        ];
        assert_eq!(res, expected);
    }
}
