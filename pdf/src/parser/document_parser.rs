use std::io::{Read, Seek};

use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFArray, PDFDictionary, PDFObject, PDFStream};
use crate::parser::cross_ref_table::{CrossRefTable, EntryInfo, EntryType};
use crate::parser::syntax::{SyntaxParser, Token};
use crate::parser::character_set::buf_to_number;

#[derive(Debug)]
pub struct DocumentParser<T: Seek + Read> {
    syntax_parser: SyntaxParser<T>,
    crosstable: CrossRefTable,
}

impl<T: Seek + Read> DocumentParser<T> {
    pub fn new(stream: T) -> PDFResult<Self> {
        let syntax_parser = SyntaxParser::try_new(stream)?;
        Ok(DocumentParser {
            syntax_parser,
            crosstable: CrossRefTable::default(),
        })
    }
    pub fn get_root_obj(&mut self) -> PDFResult<PDFObject> {
        match self.crosstable.trailer().get("Root") {
            Some(PDFObject::Indirect(rf)) => self.read_indirect_object(&rf.number()),
            _ => Err(PDFError::InvalidFileStructure(
                "faild read root".to_string(),
            )),
        }
    }

    pub fn read_indirect_object(&mut self, objnum: &u32) -> PDFResult<PDFObject> {
        if let Some(entryinfo) = self.crosstable.get_entry(objnum) {
            return self.read_indirect_object_at(entryinfo.pos());
        }
        Err(PDFError::InvalidFileStructure(format!(
            "faild found obj:{:?}",
            objnum
        )))
    }

    fn read_indirect_object_at(&mut self, pos: u64) -> PDFResult<PDFObject> {
        self.syntax_parser.seek_to(pos)?;
        let number = self.syntax_parser.next_token()?;
        if !number.is_number() {
            return Err(PDFError::InvalidFileStructure(format!(
                "read indirect expect number got {:?} ",
                number
            )));
        }
        let gen = self.syntax_parser.next_token()?;
        if !gen.is_number() {
            return Err(PDFError::InvalidFileStructure(format!(
                "read indirect expect gen as number got {:?} ",
                gen
            )));
        }
        if !self
            .syntax_parser
            .check_next_token(&Token::new_other("obj"))?
        {
            return Err(PDFError::InvalidFileStructure(
                "read indirect expect 'obj' keyword number got ".to_string(),
            ));
        }

        let obj = self.syntax_parser.read_object()?;
        Ok(obj)
    }

    fn find_startxref(&mut self) -> PDFResult<u64> {
        let search_buf_size = std::cmp::min(self.syntax_parser.size(), 4096);
        let start_pos = self.syntax_parser.size() - search_buf_size;
        self.syntax_parser.seek_to(start_pos)?;
        let pos = self.syntax_parser.find_tag(b"startxref")?;
        self.syntax_parser.seek_to(pos)?;
        if self
            .syntax_parser
            .check_next_token(&Token::new_other("startxref"))?
        {
            let num = self.syntax_parser.next_token()?;
            let start = num.to_i64()? as u64;
            Ok(start)
        } else {
            Err(PDFError::InvalidFileStructure(
                "parse startxref faild".to_string(),
            ))
        }
    }

    pub fn load_xref(&mut self) -> PDFResult<()> {
        let startxref = self.find_startxref()?;
        self.syntax_parser.seek_to(startxref)?;
        if self
            .syntax_parser
            .check_next_token(&Token::new_other("xref"))?
        {
            self.crosstable = self.load_xref_v4(startxref)?;
        } else {
            self.crosstable = self.load_xref_v5(startxref)?;
        }
        Ok(())
    }

    fn load_xref_v4(&mut self, start: u64) -> PDFResult<CrossRefTable> {
        let mut visited = Vec::new();
        visited.push(start);
        let mut res = CrossRefTable::default();
        let entries = self.parse_xref_v4()?;
        if !self
            .syntax_parser
            .check_next_token(&Token::new_other("trailer"))?
        {
            return Err(PDFError::InvalidFileStructure(
                "trailer not founded".to_string(),
            ));
        }
        let trailer: PDFDictionary = self.syntax_parser.read_object()?.try_into()?;
        res.add_entries(entries);
        let prev = trailer.get("Prev").cloned();
        res.set_trailer(trailer);

        if let Some(v) = prev {
            let mut prevpos = v.as_u64()?;
            loop {
                if visited.contains(&prevpos) {
                    break;
                }
                self.syntax_parser.seek_to(prevpos)?;
                if !self
                    .syntax_parser
                    .check_next_token(&Token::new_other("xref"))?
                {
                    return Err(PDFError::InvalidFileStructure(
                        "xref not founded".to_string(),
                    ));
                }
                let entries = self.parse_xref_v4()?;
                res.add_entries(entries);
                if !self
                    .syntax_parser
                    .check_next_token(&Token::new_other("trailer"))?
                {
                    return Err(PDFError::InvalidFileStructure(
                        "trailer not founded".to_string(),
                    ));
                }
                let trailer: PDFDictionary = self.syntax_parser.read_object()?.try_into()?;
                match trailer.get("Prev") {
                    Some(v) => {
                        prevpos = v.as_u64()?;
                        visited.push(prevpos);
                    }
                    None => break,
                }
            }
        }
        Ok(res)
    }

    fn parse_xref_v4(&mut self) -> PDFResult<Vec<EntryInfo>> {
        let mut entries = Vec::new();
        loop {
            let pos = self.syntax_parser.current_position()?;
            let start_token = self.syntax_parser.next_token()?;
            if !start_token.is_number() {
                self.syntax_parser.seek_to(pos)?;
                break;
            }
            let start = start_token.to_u32()?;
            let count = self.syntax_parser.next_token()?.to_u32()?;
            self.syntax_parser.move_next_token()?;
            let subentries = self.parse_xref_subsection_v4(start, count)?;
            entries.extend(subentries);
        }
        Ok(entries)
    }

    fn parse_xref_subsection_v4(&mut self, start: u32, count: u32) -> PDFResult<Vec<EntryInfo>> {
        let mut entries = Vec::new();
        let mut is_invalid_start = false;
        for i in 0..count {
            let line = self.syntax_parser.read_fixlen_block(20)?;
            let number = start + i;
            let pos = buf_to_number(&line[0..10]) as u64;
            let gen = buf_to_number(&line[11..16]) as u32;
            let entry_type = match line[17] {
                b'f' => EntryType::Free,
                _ => EntryType::Normal,
            };

            // fix this https://github.com/pdf-rs/pdf/issues/101
            if number == 1 && gen == 65535 && entry_type == EntryType::Free {
                is_invalid_start = true;
            }
            if is_invalid_start {
                entries.push(EntryInfo::new(number - 1, gen, pos, entry_type));
            } else {
                entries.push(EntryInfo::new(number, gen, pos, entry_type));
            }
        }
        Ok(entries)
    }

    fn load_xref_v5(&mut self, start: u64) -> PDFResult<CrossRefTable> {
        let mut visited = Vec::new();
        visited.push(start);
        let mut res = CrossRefTable::default();
        let sub = self.parse_xref_v5(start)?;
        res.merge(sub);
        if let Some(p) = res.trailer().get("Prev") {
            let mut pos = p.as_u64()?;
            loop {
                if visited.contains(&pos) {
                    break;
                }
                visited.push(pos);
                let sub = self.parse_xref_v5(pos)?;
                if let Some(sp) = sub.trailer().get("Prev") {
                    pos = sp.as_u64()?;
                    res.merge(sub);
                } else {
                    res.merge(sub);
                    break;
                }
            }
        }
        Ok(res)
    }

    fn parse_xref_v5(&mut self, start: u64) -> PDFResult<CrossRefTable> {
        self.syntax_parser.seek_to(start)?;
        let _num = self.syntax_parser.next_token()?.to_u32()?;
        let _gen = self.syntax_parser.next_token()?.to_u32()?;
        if !self
            .syntax_parser
            .check_next_token(&Token::new_other("obj"))?
        {
            return Err(PDFError::InvalidFileStructure(
                "Xref Stream need Token::PDFObj got ".to_string(),
            ));
        }

        let obj = self.syntax_parser.read_object()?;
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
        let entries = self.parse_xref_subsection_v5(&mut stream)?;
        let trailer = stream.dict();
        // TODO simplefile
        let mut res = CrossRefTable::default();
        res.set_trailer(trailer);
        res.add_entries(entries);
        Ok(res)
    }

    fn parse_xref_subsection_v5(&self, stream: &mut PDFStream) -> PDFResult<Vec<EntryInfo>> {
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
        let buffer = stream.bytes();
        let mut entries = Vec::new();
        let mut bptr = 0;
        for v in index.chunks(2) {
            let start = v[0];
            let length = v[1];
            for num in start..(start + length) {
                let t = if w[0] > 0 {
                    let mut t = 0_u32;
                    for _ in 0..w[0] {
                        t = (t << 8) + buffer[bptr] as u32;
                        bptr += 1;
                    }
                    t
                } else {
                    1_u32
                };

                let mut offset = 0;
                for _ in 0..w[1] {
                    offset = (offset << 8) + buffer[bptr] as u64;
                    bptr += 1;
                }
                let mut gen = 0;
                for _ in 0..w[2] {
                    gen = (gen << 8) + buffer[bptr] as u32;
                    bptr += 1;
                }
                match t {
                    0 => {
                        entries.push(EntryInfo::new(num as u32, gen, offset, EntryType::Free));
                    }
                    1 => {
                        entries.push(EntryInfo::new(num as u32, gen, offset, EntryType::Normal));
                    }
                    2 => {
                        let entry = EntryInfo::new(num as u32, gen, offset, EntryType::Compressed);
                        entries.push(entry);
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::Cursor;
    use std::path::PathBuf;

    fn create_memory_reader(buffer: &[u8]) -> DocumentParser<Cursor<&[u8]>> {
        let cursor = Cursor::new(buffer);
        DocumentParser::new(cursor).unwrap()
    }

    fn read_file(path: PathBuf) -> Vec<u8> {
        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    fn peek_filename(name: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push(format!("tests/resources/{}", name));
        d
    }

    #[test]
    fn test_load_cross_table() {
        let fname = peek_filename("hello_world.pdf");
        let buffer = read_file(fname);
        let mut parser = create_memory_reader(buffer.as_slice());
        parser.load_xref().unwrap();
    }

    #[test]
    fn test_parse_xref_empty() {
        let fname = peek_filename("empty_xref.pdf");
        let buffer = read_file(fname);
        let mut parser = create_memory_reader(buffer.as_slice());
        parser.load_xref().unwrap();
    }

    #[test]
    fn test_parse_start_one_xref() {
        let fname = peek_filename("xref_num_start_one.pdf");
        let buffer = read_file(fname);
        let mut parser = create_memory_reader(buffer.as_slice());
        parser.load_xref().unwrap();
        let obj = parser.read_indirect_object(&1).unwrap();
        match obj {
            PDFObject::Dictionary(dict) => assert!(dict.get("Type").is_some()),
            _ => panic!("parse filed"),
        }
    }
}
