use crate::errors::{PDFError, PDFResult};
use crate::filter::Filter;
use zune_jpeg::JpegDecoder;

#[derive(Default)]
pub struct DCTDecode {}

impl Filter for DCTDecode {
    fn decode(
        &self,
        buf: &[u8],
        _param: Option<crate::object::PDFDictionary>,
    ) -> PDFResult<Vec<u8>> {
        let mut decoder = JpegDecoder::new(buf);
        decoder
            .decode()
            .map_err(|e| PDFError::Filter(format!("DctDecode error:{:?}", e)))
    }
}
