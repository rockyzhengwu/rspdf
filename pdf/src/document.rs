use std::cell::RefCell;
use std::io::{Read, Seek};

use crate::catalog::Catalog;
use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFDictionary, PDFObject};
use crate::page::Page;
use crate::parser::document_parser::DocumentParser;

#[allow(dead_code)]
pub struct Document<T: Seek + Read> {
    parser: RefCell<DocumentParser<T>>,
    root: PDFDictionary,
    catalog: Catalog,
}

impl<T: Seek + Read> Document<T> {
    pub fn open(input: T) -> PDFResult<Self> {
        let mut parser = DocumentParser::new(input)?;
        parser.load_xref()?;
        let root = parser.get_root_obj()?.try_into()?;
        let mut doc = Document {
            parser: RefCell::new(parser),
            catalog: Catalog::default(),
            root,
        };
        doc.load_catalog()?;
        Ok(doc)
    }

    pub fn info(&self) -> PDFResult<()> {
        unimplemented!()
    }

    fn load_catalog(&mut self) -> PDFResult<()> {
        let root: PDFDictionary = self.parser.borrow_mut().get_root_obj()?.try_into()?;
        self.catalog = Catalog::try_new(root, self)?;
        Ok(())
    }

    pub fn read_indirect(&self, indirect: &PDFObject) -> PDFResult<PDFObject> {
        match indirect {
            PDFObject::Indirect(i) => self.parser.borrow_mut().read_indirect_object(&i.number()),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "need a indirect in read_indirect:{:?}",
                indirect
            ))),
        }
    }

    pub fn get_page(&self, i: &u32) -> PDFResult<Page> {
        if let Some(noderef) = self.catalog.get_page(i) {
            let data = noderef.borrow().data().to_owned();
            Page::try_new(&data, self)
        } else {
            Err(PDFError::InvalidFileStructure(format!(
                "page {:?} not existed",
                i
            )))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};

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

    #[test]
    fn test_page_tree() {
        let fname = "/home/zhengwu/Documents/books/DataStructures.pdf";
        let path = Path::new(fname);
        let buffer = read_file(path.to_path_buf());
        let cursor = create_memory_reader(buffer.as_slice());
        let doc = Document::open(cursor).unwrap();
        let page = doc.get_page(&0).unwrap();
    }
}
