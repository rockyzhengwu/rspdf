pub mod ascii_85;
pub mod ascii_hex;
pub mod ccittfax;
pub mod dct;
pub mod flate;
pub mod jbig2;
pub mod lzw;
pub mod run_length;

use crate::error::{PdfError, Result};
use crate::filter::run_length::run_length_decode;
use crate::object::dictionary::PdfDict;
use ascii_85::ascii_85_decode;
use ascii_hex::ascii_hex_decode;
use ccittfax::ccittfax_decode;
use dct::dct_decode;
use flate::flate_decode;
use jbig2::jbig2_decode;
use lzw::lzw_decode;

pub fn apply_filter(name: &str, input: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>> {
    match name {
        "AHx" | "ASCIIHexDecode" => ascii_hex_decode(input),
        "A85" | "ASCII85Decode" => ascii_85_decode(input),
        "LZW" | "LZWDecode" => lzw_decode(input, params),
        "Fl" | "FlateDecode" => flate_decode(input, params),
        "RL" | "RunLengthDecode" => run_length_decode(input),
        "DCT" | "DCTDecode" => dct_decode(input, params),
        "CCF" | "CCITTFaxDecode" => ccittfax_decode(input, params),
        "JBIG2Decode" => jbig2_decode(input, params),
        _ => Err(PdfError::Filter(format!("unimplemented:{:?}", name))),
    }
}
