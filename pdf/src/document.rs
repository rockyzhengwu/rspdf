use std::io::{Read, Seek};

use crate::errors::PDFResult;
use crate::object::{PDFDictionary, PDFObject};
use crate::parser::document_parser::DocumentParser;

#[allow(dead_code)]
pub struct Document<T: Seek + Read> {
    parser: DocumentParser<T>,
    root: PDFDictionary,
}

impl<T: Seek + Read> Document<T> {
    pub fn open(input: T) -> PDFResult<Self> {
        let mut parser = DocumentParser::new(input)?;
        parser.load_xref()?;
        let root = parser.get_root_obj()?.try_into()?;
        let doc = Document { parser, root };
        Ok(doc)
    }

    pub fn info(&self) -> PDFResult<()> {
        unimplemented!()
    }

    pub fn load_pages(&mut self) {}

    pub fn read_indirect(&self, indirect: &PDFObject) -> PDFResult<PDFObject> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::Cursor;
    use std::path::PathBuf;

    fn create_memory_reader(buffer: &[u8]) -> Cursor<&[u8]> {
        Cursor::new(buffer)
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
    fn test_document() {
        let fname = peek_filename("hello_world.pdf");
        let buffer = read_file(fname);
        let cursor = create_memory_reader(buffer.as_slice());
        let doc = Document::open(cursor).unwrap();
    }
}
