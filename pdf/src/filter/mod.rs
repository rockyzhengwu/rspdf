use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

use ascii85_decode::ASCII85Decode;
use asciihex_decode::ASCIIHexDecode;
use ccittfax_decode::CCITTFaxDecode;
use dct_decode::DCTDecode;
use flate_decode::FlateDecode;

pub trait Filter {
    fn decode(&self, buf: &[u8], param: Option<&PDFObject>) -> PDFResult<Vec<u8>>;
}

pub fn new_filter(name: &str) -> PDFResult<Box<dyn Filter>> {
    match name {
        "FlateDecode" => Ok(Box::<FlateDecode>::default()),
        "ASCII85Decode" | "A85" => Ok(Box::<ASCII85Decode>::default()),
        "ASCIIHexDecode" => Ok(Box::<ASCIIHexDecode>::default()),
        "DCTDecode" => Ok(Box::<DCTDecode>::default()),
        "CCITTFaxDecode" => Ok(Box::<CCITTFaxDecode>::default()),
        _ => Err(PDFError::Filter(format!("Filter {:?} not supported", name))),
    }
}

pub mod ascii85_decode;
pub mod asciihex_decode;
pub mod ccittfax_decode;
pub mod dct_decode;
pub mod flate_decode;
