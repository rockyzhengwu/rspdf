use flate2::bufread::ZlibDecoder;
use std::io::Read;

use crate::errors::{PDFError, PDFResult};
use crate::filter::Filter;
use crate::object::PDFDictionary;

#[derive(Default)]
pub struct FlateDecode {}

// TODO filter params and more filter

impl Filter for FlateDecode {
    fn decode(&self, buffer: &[u8], _params: Option<PDFDictionary>) -> PDFResult<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(buffer);
        let mut out: Vec<u8> = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|_| PDFError::Filter("Zlib deocde error".to_string()))?;
        Ok(out)
    }
}
