use std::collections::HashMap;
use std::i64;
use std::io::{Read, Seek};

use log::warn;

use crate::errors::{PDFError, PDFResult};
use crate::lexer::Tokenizer;
use crate::object::{
    PDFArray, PDFDictionary, PDFIndirect, PDFName, PDFNumber, PDFObject, PDFStream, PDFString,
};

use crate::token::Token;
use crate::xref::{XRefEntry, XRefEntryTable, XRefEntryType};

pub struct Reader<T: Seek + Read> {
    tokenizer: Tokenizer<T>,
}

impl<T: Seek + Read> Reader<T> {
    pub fn new(file: T) -> Self {
        let tokenizer: Tokenizer<T> = Tokenizer::new(file);
        Reader { tokenizer }
    }

    pub fn fetch_object(&mut self, entry: &XRefEntry) -> PDFResult<PDFObject> {
        let offset = entry.offset();
        self.tokenizer.seek(offset as u64)?;

        let _number = self.parse_obj()?;
        let _gen = self.parse_obj()?;

        let token = self.tokenizer.next_token()?;

        if token != Token::PDFObj {
            return Err(PDFError::InvalidFileStructure(format!(
                "Featch object {:?} end expected Token::PDFObj got {:?}",
                entry, token
            )));
        }
        let obj = self.parse_obj()?;
        Ok(obj)
    }

    pub fn read_xref(&mut self) -> PDFResult<(PDFObject, XRefEntryTable)> {
        let start = self.tokenizer.find_start_xref()?;
        self.tokenizer.seek(start)?;
        if self.tokenizer.check_next_type(&Token::PDFXRef)? {
            self.parse_xref_table(start)
        } else {
            self.parse_xref_stream()
        }
    }

    fn parse_xref_table(&mut self, start: u64) -> PDFResult<(PDFObject, XRefEntryTable)> {
        let mut entries: XRefEntryTable = HashMap::new();
        let mut stack = Vec::new();
        stack.push(start);
        let mut step = 0;
        let mut trailer = PDFObject::Dictionary(HashMap::new());
        while let Some(pos) = stack.pop() {
            self.tokenizer.seek(pos)?;
            let sub_entries = self.parse_xref_subsection_table()?;
            entries.extend(sub_entries);
            let obj = self.parse_obj()?;
            let prev = obj.get_value_as_i64("Prev");
            if step == 0 {
                trailer = obj;
            }
            step += 1;
            if let Some(Ok(p)) = prev {
                stack.push(p as u64)
            }
        }

        Ok((trailer, entries))
    }

    fn parse_xref_subsection_table(&mut self) -> PDFResult<XRefEntryTable> {
        // TODO mutil section
        let mut entries = HashMap::new();
        let mut token = self.tokenizer.next_token()?;
        assert_eq!(token, Token::PDFXRef);
        loop {
            token = self.tokenizer.next_token()?;
            if token == Token::PDFTrailer || token == Token::PDFEof {
                break;
            }
            let start: i64 = token.as_i64()?;
            let count: i64 = self.tokenizer.next_token()?.as_i64()?;
            let mut fix_num = start;
            for num in start..(start + count) {
                let mut token = self.tokenizer.next_token()?;
                if token == Token::PDFTrailer {
                    break;
                }
                let offset: i64 = token.as_i64()?;
                token = self.tokenizer.next_token()?;
                let gen: i64 = token.as_i64()?;
                token = self.tokenizer.next_token()?;
                let xtype = match token.as_string()?.as_str() {
                    "f" => XRefEntryType::XRefEntryFree,
                    "n" => XRefEntryType::XRefEntryUncompressed,
                    _ => {
                        return Err(PDFError::InvalidFileStructure(format!(
                            "Xref Entry type mut be 'n' or 'f' got {:?}",
                            token
                        )))
                    }
                };
                // fix this https://github.com/pdf-rs/pdf/issues/101
                if num == 1 && gen == 65535 && matches!(xtype, XRefEntryType::XRefEntryFree) {
                    fix_num = 0;
                }
                let entry = XRefEntry::new(fix_num, offset, gen, xtype);
                entries.insert((fix_num, gen), entry);
                fix_num += 1;
            }
        }
        Ok(entries)
    }

    fn parse_xref_stream(&mut self) -> PDFResult<(PDFObject, XRefEntryTable)> {
        // TODO for loop to parse all xref stream
        let _num: i64 = self.tokenizer.next_token()?.as_i64()?;
        // gen
        let _gen: i64 = self.tokenizer.next_token()?.as_i64()?;
        // obj
        let token = self.tokenizer.next_token()?;
        if token != Token::PDFObj {
            return Err(PDFError::InvalidFileStructure(format!(
                "Xref Stream need Token::PDFObj got {:?}",
                token
            )));
        }

        let obj = self.parse_obj()?;
        let mut stream: PDFStream = obj.try_into()?;
        match stream.attribute("Type") {
            Some(obj) => {
                if "XRef" != obj.as_string()? {
                    return Err(PDFError::InvalidFileStructure(format!(
                        "XRef Stream need Type 'XRef' got {:?}",
                        obj
                    )));
                }
            }
            None => {
                return Err(PDFError::InvalidFileStructure(
                    "XRef Stream Type dosn't exist".to_string(),
                ))
            }
        }
        let entries = self.parse_xref_subsection_stream(&mut stream)?;
        let trailer = PDFObject::Dictionary(stream.dict());
        Ok((trailer, entries))
    }

    fn parse_xref_subsection_stream(
        &mut self,
        stream: &mut PDFStream,
    ) -> PDFResult<XRefEntryTable> {
        let wobj: PDFArray = stream
            .attribute("W")
            .ok_or(PDFError::InvalidSyntax(
                "W dos'nt in xref stream".to_string(),
            ))?
            .to_owned()
            .try_into()?;
        let mut w = Vec::new();
        for v in wobj {
            w.push(v.as_i64()?);
        }

        let indexobj: PDFArray = stream
            .attribute("Index")
            .ok_or(PDFError::InvalidFileStructure(
                "Index dos'nt in xref stream".to_string(),
            ))?
            .to_owned()
            .try_into()?;

        let mut index = Vec::new();
        for v in indexobj {
            index.push(v.as_i64()?);
        }
        let buffer = match stream.length() {
            Some(l) => match l.as_i64() {
                Ok(ll) => self.read_stream_content(stream, ll as usize)?,
                Err(e) => {
                    warn!("Xref Stream Length not number got:{:?}", e);
                    self.read_stream_content_unitl_end(stream)?
                }
            },
            None => {
                warn!("Xref Stream Length not exists");
                self.read_stream_content_unitl_end(stream)?
            }
        };

        stream.set_buffer(buffer);
        let buffer = stream.bytes();
        let mut entries = HashMap::new();
        let mut bptr = 0;
        for v in index.chunks(2) {
            let start = v[0];
            let length = v[1];
            for num in start..(start + length) {
                let t = if w[0] > 0 {
                    let mut t = 0_i64;
                    for _ in 0..w[0] {
                        t = (t << 8) + buffer[bptr] as i64;
                        bptr += 1;
                    }
                    t
                } else {
                    1_i64
                };

                let mut offset = 0;
                for _ in 0..w[1] {
                    offset = (offset << 8) + buffer[bptr] as i64;
                    bptr += 1;
                }
                let mut gen = 0;
                for _ in 0..w[2] {
                    gen = (gen << 8) + buffer[bptr] as i64;
                    bptr += 1;
                }
                match t {
                    0 => {
                        entries.insert(
                            (num, gen),
                            XRefEntry::new(num, offset, gen, XRefEntryType::XRefEntryFree),
                        );
                    }
                    1 => {
                        entries.insert(
                            (num, gen),
                            XRefEntry::new(num, offset, gen, XRefEntryType::XRefEntryUncompressed),
                        );
                    }
                    2 => {
                        let mut entry =
                            XRefEntry::new(num, gen, 0, XRefEntryType::XRefEntryCompressed);
                        entry.set_stream_offset(offset);
                        entries.insert((num, gen), entry);
                    }
                    _ => {
                        return Err(PDFError::InvalidFileStructure(format!(
                            "Xref Entry type must 1,2 or 3 got :{}",
                            t
                        )));
                    }
                }
            }
        }

        Ok(entries)
    }

    pub fn parse_obj(&mut self) -> PDFResult<PDFObject> {
        let token = self.tokenizer.next_token()?;
        match token {
            Token::PDFOpenArray => self.parse_array(),
            Token::PDFOpenDict => self.parse_dict(),
            Token::PDFHexString(s) => Ok(PDFObject::String(PDFString::HexString(s))),
            Token::PDFLiteralString(s) => Ok(PDFObject::String(PDFString::Literial(s))),
            Token::PDFName(s) => Ok(PDFObject::Name(PDFName::new(s.as_str()))),
            Token::PDFNumber(v) => Ok(PDFObject::Number(PDFNumber::Integer(v))),
            Token::PDFTrue => Ok(PDFObject::Bool(true)),
            Token::PDFFalse => Ok(PDFObject::Bool(false)),
            Token::PDFReal(v) => Ok(PDFObject::Number(PDFNumber::Real(v))),
            Token::PDFNull => Ok(PDFObject::Null),
            Token::PDFIndirect(num, gen) => Ok(PDFObject::Indirect(PDFIndirect::new(
                num.to_owned() as u32,
                gen.to_owned() as u16,
            ))),
            _ => Err(PDFError::InvalidSyntax(format!(
                "Token {:?} not a invalid PDFObject starter",
                token
            ))),
        }
    }

    fn parse_dict(&mut self) -> PDFResult<PDFObject> {
        let mut token = self.tokenizer.next_token()?;
        let mut dictionary = PDFDictionary::new();
        while token != Token::PDFCloseDict {
            match token {
                Token::PDFCloseDict => {
                    // no need to do this
                    break;
                }
                Token::PDFName(key) => {
                    let val = self.parse_obj()?;
                    dictionary.insert(key, val);
                }
                _ => {
                    return Err(PDFError::InvalidSyntax(format!(
                        "PDFDictionary key must be Token::PDFName got :{:?}",
                        token
                    )));
                }
            }
            // Key
            token = self.tokenizer.next_token()?;
        }

        if token != Token::PDFCloseDict {
            return Err(PDFError::InvalidSyntax(format!(
                "PDFDictionary need Token::PDFCloseDict got :{:?}",
                token
            )));
        }
        let ofs = self.tokenizer.offset()?;
        token = self.tokenizer.next_token()?;

        if token == Token::PDFStream {
            return self.parse_stream(PDFObject::Dictionary(dictionary));
        } else {
            self.tokenizer.seek(ofs)?;
        }
        Ok(PDFObject::Dictionary(dictionary))
    }

    pub fn parse_array(&mut self) -> PDFResult<PDFObject> {
        let mut array: Vec<PDFObject> = Vec::new();
        loop {
            if self.tokenizer.check_next_type(&Token::PDFCloseArray)? {
                self.tokenizer.next_token()?;
                break;
            }
            let val = self.parse_obj()?;
            array.push(val)
        }
        Ok(PDFObject::Arrray(array))
    }

    pub fn parse_stream(&mut self, obj: PDFObject) -> PDFResult<PDFObject> {
        // token is next stream
        let dict: PDFDictionary = obj.try_into()?;
        let offset = self.tokenizer.skip_white()?;
        let stream = PDFObject::Stream(PDFStream::new(offset, dict));
        Ok(stream)
    }

    pub fn read_stream_content(&mut self, stream: &PDFStream, length: usize) -> PDFResult<Vec<u8>> {
        let offset = stream.offset();
        self.tokenizer.seek(offset)?;
        let mut buffer = vec![0; length];
        self.tokenizer.peek_buffer(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_stream_content_unitl_end(&mut self, stream: &PDFStream) -> PDFResult<Vec<u8>> {
        let offset = stream.offset();
        self.tokenizer.seek(offset)?;
        let buffer = self.tokenizer.read_unitil(b"endstream")?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::object::PDFObject;
    use crate::xref::{XRefEntry, XRefEntryType};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Cursor;
    use std::path::PathBuf;

    use crate::reader::Reader;
    fn create_memory_reader(buffer: &[u8]) -> Reader<Cursor<&[u8]>> {
        let cursor = Cursor::new(buffer);
        Reader::new(cursor)
    }

    fn create_file_reader(path: PathBuf) -> Reader<File> {
        let file = File::open(path).unwrap();
        Reader::new(file)
    }

    fn peek_filename(name: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push(format!("tests/resources/{}", name));
        d
    }

    #[test]
    fn test_parse_nest_dict() {
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
        let mut parser = create_memory_reader(content);
        let obj = parser.parse_obj().unwrap();
        assert!(matches!(obj, PDFObject::Dictionary(_)));
        let resource = obj.get_value("Resources").unwrap().to_owned();
        assert!(matches!(resource, PDFObject::Dictionary(_)))
    }

    #[test]
    fn test_parse_xref_table() {
        let fname = peek_filename("hello_world.pdf");
        let mut parser = create_file_reader(fname);
        let (trailer, entries) = parser.read_xref().unwrap();
        assert_eq!(entries.len(), 7);
        assert_eq!(trailer.get_value("Size").unwrap().as_i64().unwrap(), 7);
    }

    #[test]
    fn test_parse_xref_empty() {
        let fname = peek_filename("empty_xref.pdf");
        let mut parser = create_file_reader(fname);
        let (_trailer, entries) = parser.read_xref().unwrap();
        assert_eq!(entries.len(), 7);
    }

    #[test]
    fn test_parse_invalid_xref() {
        let fname = peek_filename("xref_num_start_one.pdf");
        let mut parser = create_file_reader(fname);
        let (_trailer, entries) = parser.read_xref().unwrap();
        let mut expected_entries = HashMap::new();
        expected_entries.insert(
            (0, 65535),
            XRefEntry::new(0, 0, 65535, XRefEntryType::XRefEntryFree),
        );
        expected_entries.insert(
            (1, 0),
            XRefEntry::new(1, 17, 0, XRefEntryType::XRefEntryUncompressed),
        );
        expected_entries.insert(
            (2, 0),
            XRefEntry::new(2, 66, 0, XRefEntryType::XRefEntryUncompressed),
        );
        expected_entries.insert(
            (3, 0),
            XRefEntry::new(3, 122, 0, XRefEntryType::XRefEntryUncompressed),
        );
        expected_entries.insert(
            (4, 0),
            XRefEntry::new(4, 209, 0, XRefEntryType::XRefEntryUncompressed),
        );
        expected_entries.insert(
            (5, 0),
            XRefEntry::new(5, 314, 0, XRefEntryType::XRefEntryUncompressed),
        );
        expected_entries.insert(
            (6, 0),
            XRefEntry::new(6, 445, 0, XRefEntryType::XRefEntryUncompressed),
        );
        assert_eq!(entries, expected_entries);
    }
}
