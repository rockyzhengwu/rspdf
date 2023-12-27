use flate2::bufread::ZlibDecoder;
use std::io::Read;

use log::warn;

use crate::errors::PDFResult;
use crate::filter::Filter;
use crate::object::PDFDictionary;

#[derive(Default)]
pub struct FlateDecode {}

impl Filter for FlateDecode {
    fn decode(&self, buffer: &[u8], _params: Option<PDFDictionary>) -> PDFResult<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(buffer);
        let mut out: Vec<u8> = Vec::new();
        let mut buf: [u8; 1] = [0; 1];
        // read until error, if has error will ignore, maybe some elegant method
        loop {
            match decoder.read(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }

                    out.push(buf[0]);
                }
                Err(e) => {
                    warn!("FlateDecode error :{:?}", e);
                    break;
                }
            }
        }
        Ok(out)
    }
}
