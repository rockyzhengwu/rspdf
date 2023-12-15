use std::fmt;
use std::i64;

use crate::errors::{PDFError, PDFResult};

#[derive(PartialEq, Clone)]
pub enum Token {
    PDFOpenArray,
    PDFCloseArray,
    PDFOpenDict,
    PDFCloseDict,
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

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::PDFOpenArray => write!(f, "["),
            Token::PDFCloseArray => write!(f, "]"),
            Token::PDFOpenDict => write!(f, "<<"),
            Token::PDFCloseDict => write!(f, ">>"),
            Token::PDFHexString(v) => write!(f, "Hexstring({:?})", String::from_utf8_lossy(v)),
            Token::PDFLiteralString(v) => write!(f, "Literal({:?})", String::from_utf8_lossy(v)),
            Token::PDFName(s) => write!(f, "Name({:?})", s),
            Token::PDFRef => write!(f, "R"),
            Token::PDFTrue => write!(f, "Bool(true)"),
            Token::PDFFalse => write!(f, "Bool(false)"),
            Token::PDFNull => write!(f, "Null"),
            Token::PDFObj => write!(f, "obj"),
            Token::PDFEndObj => write!(f, "endobj"),
            Token::PDFStream => write!(f, "stream"),
            Token::PDFEndStream => write!(f, "endstream"),
            Token::PDFXRef => write!(f, "xref"),
            Token::PDFStartXRef => write!(f, "startxref"),
            Token::PDFTrailer => write!(f, "trailer"),
            Token::PDFNumber(i) => write!(f, "Number({})", i),
            Token::PDFReal(v) => write!(f, "Real({})", v),
            Token::PDFIndirect(n, g) => write!(f, "Indirect({},{})", n, g),
            Token::PDFOther(v) => write!(f, "Other({:?})", String::from_utf8_lossy(v)),
            Token::PDFEof => write!(f, "Eof"),
        }
    }
}
