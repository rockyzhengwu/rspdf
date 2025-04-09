use crate::error::{PdfError, Result};
use crate::object::dictionary::PdfDict;

use std::io::Write;

pub fn jbig2_decode(buf: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>> {
    return Err(PdfError::Filter(
        "Jbig2_decoder is not implemented".to_string(),
    ));
    //unimplemented!("Jbig2_decoder unimplemented")
}
