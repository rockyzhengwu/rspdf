use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::object::PDFDictionary;

pub struct Page {}

pub mod content_parser;

impl Page {
    pub fn try_new<T: Seek + Read>(pagedict: &PDFDictionary, doc: &Document<T>) -> PDFResult<Self> {
        println!("{:?}", pagedict);
        unimplemented!()
    }
}
