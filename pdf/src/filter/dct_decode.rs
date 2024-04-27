use crate::errors::{PDFError, PDFResult};
use crate::filter::Filter;
use crate::object::PDFObject;
use zune_jpeg::JpegDecoder;

#[derive(Default)]
pub struct DCTDecode {}

impl Filter for DCTDecode {
    fn decode(&self, buf: &[u8], _param: Option<&PDFObject>) -> PDFResult<Vec<u8>> {
        let mut decoder = JpegDecoder::new(buf);
        decoder
            .decode()
            .map_err(|e| PDFError::Filter(format!("DctDecode error:{:?}", e)))
    }
}
