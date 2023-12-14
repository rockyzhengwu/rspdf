
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFDictionary;
use ascii85_decode::ASCII85Decode;
use flate_decode::FlateDecode;

pub trait Filter {
    fn decode(&self, buf: &[u8], param: Option<PDFDictionary>) -> PDFResult<Vec<u8>>;
}

pub fn new_filter(name: &str) -> PDFResult<Box<dyn Filter>> {
    match name {
        "FlateDecode" => Ok(Box::<FlateDecode>::default()),
        "ASCII85Decode" => Ok(Box::<ASCII85Decode>::default()),
        _ => Err(PDFError::Filter(format!("Filter {:?} not supported", name))),
    }
}

pub mod ascii85_decode;
pub mod flate_decode;
