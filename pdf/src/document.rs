use std::cell::RefCell;
use std::io::{Read, Seek};

use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use crate::page::{PageRef, PageTree};
use crate::reader::Reader;
use crate::xref::XRef;

#[allow(dead_code)]
pub struct Document<T: Seek + Read> {
    version: f32,
    reader: RefCell<Reader<T>>,
    xref: XRef,
    page_tree: Option<PageTree>,
}

impl<T: Seek + Read> Document<T> {
    pub fn open(input: T) -> PDFResult<Self> {
        let mut reader = RefCell::new(Reader::new(input));
        let (trailer, entries) = reader.get_mut().read_xref()?;

        let xref = XRef::try_new(trailer, entries)?;
        // xref, reader
        // build page_tree

        let mut doc = Document {
            version: 1.17,
            reader,
            xref,
            page_tree: None,
        };
        let page_tree = PageTree::try_new(&doc)?;
        doc.page_tree = page_tree;

        Ok(doc)
    }

    pub fn info(&self) -> PDFResult<()> {
        unimplemented!()
    }

    pub fn catalog(&self) -> PDFObject {
        let root = self.xref.root().unwrap();
        match root {
            PDFObject::Indirect(_) => self.read_indirect(root).unwrap(),
            PDFObject::Dictionary(_) => root.to_owned(),
            _ => panic!("Root not Indirect or Dictionary"),
        }
    }

    pub fn page(&self, number: u32) -> PDFResult<PageRef> {
        self.page_tree
            .as_ref()
            .unwrap()
            .get_page(number)
            .ok_or(PDFError::InvalidFileStructure(format!(
                "page {:?}not exists",
                number
            )))
    }

    pub fn add_page(&self) -> PDFResult<()> {
        unimplemented!()
    }

    pub fn create() -> PDFResult<Self> {
        unimplemented!()
    }

    pub fn read_indirect(&self, indirect: &PDFObject) -> PDFResult<PDFObject> {
        let entry = self.xref.indirect_entry(indirect)?;
        self.reader.borrow_mut().fetch_object(entry)
    }
}
