use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::object::PDFObject;

#[derive(Debug)]
pub struct Separation {}

impl Separation {
    pub fn try_new<T: Seek + Read>(obj: &PDFObject, doc: Document<T>) -> PDFResult<Self> {
        println!("{:?}", obj);

        unimplemented!()
    }
}
