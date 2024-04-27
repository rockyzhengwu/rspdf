use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::catalog::Catalog;
use crate::errors::{PDFError, PDFResult};
use crate::font::pdf_font::Font;
use crate::object::{PDFDictionary, PDFObject};
use crate::page::Page;
use crate::parser::cross_ref_table::CrossRefTable;
use crate::parser::document_parser::DocumentParser;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Document<T: Seek + Read> {
    parser: RefCell<DocumentParser<T>>,
    catalog: Catalog,
    fonts: RefCell<HashMap<String, Font>>,
    crosstable: CrossRefTable,
}

impl<T: Seek + Read> Document<T> {
    pub fn open(input: T) -> PDFResult<Self> {
        let mut parser = DocumentParser::new(input)?;
        let crosstable = parser.load_xref()?;
        let mut doc = Document {
            parser: RefCell::new(parser),
            catalog: Catalog::default(),
            fonts: RefCell::new(HashMap::new()),
            crosstable,
        };
        doc.load_catalog()?;
        Ok(doc)
    }

    pub fn info(&self) -> PDFResult<()> {
        unimplemented!()
    }

    pub fn get_font(&self, name: &str) -> Option<Font> {
        //self.fonts.borrow().get(name).cloned()
        None
    }

    pub fn add_font(&self, name: &str, font: Font) {
        self.fonts.borrow_mut().insert(name.to_string(), font);
    }

    pub fn page_count(&self) -> PDFResult<u32> {
        self.catalog.page_count()
    }

    pub fn get_root_obj(&mut self) -> PDFResult<PDFObject> {
        if let Some(root) = self.crosstable.trailer().get("Root") {
            self.read_indirect(root)
        } else {
            Err(PDFError::InvalidFileStructure(
                "faild read root".to_string(),
            ))
        }
    }

    fn load_catalog(&mut self) -> PDFResult<()> {
        let root: PDFDictionary = self.get_root_obj()?.try_into()?;
        self.catalog = Catalog::try_new(root, self)?;
        Ok(())
    }

    pub fn read_indirect(&self, indirect: &PDFObject) -> PDFResult<PDFObject> {
        match indirect {
            PDFObject::Indirect(i) => {
                if let Some(entryinfo) = self.crosstable.get_entry(&i.number()) {
                    let mut obj = self
                        .parser
                        .borrow_mut()
                        .read_indirect_object(entryinfo.pos())?;
                    match obj {
                        // TODO fix this ugly method
                        PDFObject::Stream(_) => {
                            if let Some(o) = obj.get_value("DecodeParms") {
                                let o = self.get_object_without_indriect(o)?;
                                obj.set_value("DecodeParms", o)?;
                            }
                        }
                        _ => {
                            //
                        }
                    }
                    Ok(obj)
                } else {
                    Err(PDFError::InvalidFileStructure(format!(
                        "faild found obj:{:?}",
                        i
                    )))
                }
            }
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "need a indirect in read_indirect:{:?}",
                indirect
            ))),
        }
    }
    pub fn get_object_without_indriect(&self, obj: &PDFObject) -> PDFResult<PDFObject> {
        match obj {
            PDFObject::Indirect(_) => self.read_indirect(obj),
            _ => Ok(obj.to_owned()),
        }
    }

    pub fn get_page(&self, i: &u32) -> PDFResult<Page<T>> {
        if let Some(noderef) = self.catalog.get_page(i) {
            Page::try_new(i, noderef.clone(), self)
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
        let _doc = Document::open(cursor).unwrap();
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
