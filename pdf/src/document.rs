use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;

use crate::catalog::Catalog;
use crate::errors::{PDFError, PDFResult};
use crate::font::Font;
use crate::object::{PDFDictionary, PDFObject};
use crate::page::Page;
use crate::parser::document_parser::DocumentParser;

#[derive(Debug)]
pub struct Document<T: Seek + Read> {
    parser: RefCell<DocumentParser<T>>,
    root: PDFDictionary,
    catalog: Catalog,
    fonts: RefCell<HashMap<String, Rc<Font>>>,
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
            fonts: RefCell::new(HashMap::new()),
        };
        doc.load_catalog()?;
        Ok(doc)
    }

    pub fn info(&self) -> PDFResult<()> {
        unimplemented!()
    }

    pub fn get_font(&self, name: &str) -> Option<Rc<Font>> {
        self.fonts.borrow().get(name).cloned()
    }

    pub fn add_font(&self, name: &str, font: Rc<Font>) {
        self.fonts.borrow_mut().insert(name.to_string(), font);
    }

    pub fn page_count(&self) -> PDFResult<u32> {
        self.catalog.page_count()
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

    pub fn get_page(&self, i: &u32) -> PDFResult<Page<T>> {
        if let Some(noderef) = self.catalog.get_page(i) {
            Page::try_new(noderef.clone(), self)
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
    use crate::device::text::TextDevice;
    use std::fs::File;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

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
        let textdevice = Rc::new(RefCell::new(TextDevice::new()));
        page.display(textdevice).unwrap();
    }
}
