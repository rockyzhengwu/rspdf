use std::fmt::Display;

use crate::error::Result;

#[derive(Debug, PartialEq, Clone)]
pub struct PdfName {
    name: String,
}

impl PdfName {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn from_buffer(buf: Vec<u8>) -> Result<Self> {
        let name = String::from_utf8(buf).unwrap();
        Ok(PdfName { name })
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Display for PdfName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
