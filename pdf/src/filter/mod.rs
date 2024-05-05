use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

use ascii85_decode::ascii85_decode;
use asciihex_decode::asciihex_decode;
use ccittfax_decode::fax_decode;
use dct_decode::dct_decode;
use flate_decode::flate_decode;

pub fn decode(name: &str, buf: &[u8], param: Option<&PDFObject>) -> PDFResult<Vec<u8>> {
    match name {
        "FlateDecode" => flate_decode(buf),
        "ASCII85Decode" | "A85" => ascii85_decode(buf),
        "ASCIIHexDecode" => asciihex_decode(buf),
        "DCTDecode" => dct_decode(buf),
        "CCITTFaxDecode" => fax_decode(buf, param),
        _ => Err(PDFError::Filter(format!("Filter {:?} not supported", name))),
    }
}

pub mod ascii85_decode;
pub mod asciihex_decode;
pub mod ccittfax_decode;
pub mod dct_decode;
pub mod flate_decode;
