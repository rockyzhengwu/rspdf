use crate::error::{PdfError, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct PdfLiteral {
    bytes: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PdfHexString {
    bytes: Vec<u8>,
}

impl PdfLiteral {
    pub fn new(bytes: Vec<u8>) -> Self {
        PdfLiteral { bytes }
    }
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl PdfHexString {
    pub fn new(bytes: Vec<u8>) -> Self {
        PdfHexString { bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn raw_bytes(&self) -> Result<Vec<u8>> {
        self.bytes
            .chunks(2)
            .map(|pair| Ok(hex_to_u8(pair[0])? << 4 | hex_to_u8(pair[1])?))
            .collect()
    }
}

fn hex_to_u8(c: u8) -> Result<u8> {
    match c {
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(PdfError::Object(format!(
            "PdfHexString hex to bytes error:{:?}",
            c
        ))),
    }
}
