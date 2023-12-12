use std::i64;

use crate::errors::{PDFError, PDFResult};

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    PDFOpenArray,
    PDFCloseArray,
    PDFOpenDict,
    PDFCloseDict,
    PDFOpenBrace,
    PDFCloseBrace,
    PDFHexString(Vec<u8>),
    PDFLiteralString(Vec<u8>),
    PDFName(String),
    PDFRef,
    PDFTrue,
    PDFFalse,
    PDFNull,
    PDFObj,
    PDFEndObj,
    PDFStream,
    PDFEndStream,
    PDFXRef,
    PDFStartXRef,
    PDFTrailer,
    PDFNumber(i64),
    PDFReal(f64),
    PDFIndirect(i64, i64),
    PDFOther(Vec<u8>),
    PDFEof,
}

impl Token {
    pub fn as_string(&self) -> PDFResult<String> {
        match self {
            Token::PDFOther(buf) => Ok(String::from_utf8_lossy(buf.as_slice()).to_string()),
            Token::PDFName(name) => Ok(name.to_string()),
            Token::PDFLiteralString(s) => Ok(String::from_utf8_lossy(s).to_string()),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} can't convert to String ",
                self,
            ))),
        }
    }

    pub fn as_i64(&self) -> PDFResult<i64> {
        match self {
            Token::PDFNumber(d) => Ok(d.to_owned()),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} can't convert to i64",
                self
            ))),
        }
    }
    pub fn as_f64(&self) -> PDFResult<f64> {
        match self {
            Token::PDFReal(r) => Ok(r.to_owned()),
            Token::PDFNumber(n) => Ok(n.to_owned() as f64),
            _ => Err(PDFError::TokenConvertFailure(format!(
                "{:?} can't convert to i64",
                self
            ))),
        }
    }
}
