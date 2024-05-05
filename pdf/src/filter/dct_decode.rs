use crate::errors::{PDFError, PDFResult};
use zune_jpeg::JpegDecoder;

pub fn dct_decode(buf: &[u8]) -> PDFResult<Vec<u8>> {
    let mut decoder = JpegDecoder::new(buf);
    decoder
        .decode()
        .map_err(|e| PDFError::Filter(format!("DctDecode error:{:?}", e)))
}
