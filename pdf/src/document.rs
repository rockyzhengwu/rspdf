use std::cell::RefCell;
use std::io::{Read, Seek};

use log::debug;

use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use crate::page::{PageRef, PageTree};
use crate::reader::Reader;
use crate::xref::XRef;

#[allow(dead_code)]
pub struct Document<T: Seek + Read> {
    version: f32,
    xref: XRef<T>,
    page_tree: Option<PageTree>,
}

impl<T: Seek + Read> Document<T> {
    pub fn open(input: T) -> PDFResult<Self> {
        let mut reader = RefCell::new(Reader::new(input));
        debug!("Read XRef");
        let (trailer, entries) = reader.get_mut().read_xref()?;

        let xref = XRef::try_new(reader, trailer, entries)?;
        // xref, reader
        // build page_tree

        debug!("Create Page True");
        let page_tree = PageTree::try_new(&xref)?;
        let mut doc = Document {
            version: 1.17,
            xref,
            page_tree: None,
        };
        doc.page_tree = page_tree;

        Ok(doc)
    }

    pub fn info(&self) -> PDFResult<()> {
        unimplemented!()
    }

    pub fn page_count(&self) -> i64 {
        match self.page_tree {
            Some(ref pt) => pt.page_count(),
            None => 0,
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
        self.xref.fetch_object(indirect)
    }
}
