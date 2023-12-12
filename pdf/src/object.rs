use std::collections::HashMap;

use crate::errors::{PDFError, PDFResult};
use crate::filter::new_filter;

#[derive(Debug, Clone)]
pub struct PDFIndirect {
    number: i64,
    gen: i64,
}

impl PDFIndirect {
    pub fn new(number: i64, gen: i64) -> Self {
        PDFIndirect { number, gen }
    }

    pub fn number(&self) -> i64 {
        self.number
    }

    pub fn gen(&self) -> i64 {
        self.gen
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct PDFName {
    name: String,
}

impl PDFName {
    pub fn new(s: &str) -> Self {
        // TODO: handle #hexdecimal format in string
        PDFName { name: s.to_owned() }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ToString for PDFName {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

pub type PDFDictionary = HashMap<PDFName, PDFObject>;

#[derive(Debug, Clone)]
pub enum PDFString {
    HexString(Vec<u8>),
    Literial(Vec<u8>),
}
impl PDFString {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::HexString(v) => v.as_slice(),
            Self::Literial(v) => v.as_slice(),
        }
    }
}

impl ToString for PDFString {
    fn to_string(&self) -> String {
        match self {
            PDFString::HexString(hex) => String::from_utf8_lossy(hex.as_slice()).to_string(),
            PDFString::Literial(literial) => {
                String::from_utf8_lossy(literial.as_slice()).to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PDFStream {
    offset: u64,
    length: i64,
    dict: PDFDictionary,
    buffer: Vec<u8>,
}

impl PDFStream {
    pub fn new(offset: u64, length: i64, dict: PDFDictionary, buffer: Vec<u8>) -> Self {
        PDFStream {
            offset,
            length,
            dict,
            buffer,
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn length(&self) -> i64 {
        self.length
    }

    pub fn attribute(&self, name: &str) -> Option<&PDFObject> {
        self.dict.get(&PDFName::new(name))
    }

    pub fn dict(&self) -> PDFDictionary {
        self.dict.clone()
    }

    pub fn bytes(&self) -> Vec<u8> {
        let filter = self.attribute("Filter");
        let mut filters = Vec::new();
        match filter {
            Some(PDFObject::Name(name)) => {
                filters.push(name.to_string());
            }
            Some(PDFObject::Arrray(arr)) => {
                for a in arr.iter() {
                    filters.push(a.as_string().unwrap());
                }
            }
            _ => {}
        }
        let mut buffer = self.buffer.to_owned();
        for fname in filters {
            let filter = new_filter(fname.as_str()).unwrap();
            buffer = filter.decode(buffer.as_slice(), None).unwrap();
        }
        buffer
    }
}

#[derive(Debug, Clone)]
pub enum PDFNumber {
    Real(f64),
    Integer(i64),
}

impl PDFNumber {
    pub fn as_f64(&self) -> f64 {
        match *self {
            PDFNumber::Real(r) => r,
            PDFNumber::Integer(i) => i as f64,
        }
    }

    pub fn as_i64(&self) -> i64 {
        match *self {
            PDFNumber::Real(r) => r as i64,
            PDFNumber::Integer(i) => i,
        }
    }
}

pub type PDFArray = Vec<PDFObject>;

#[derive(Debug, Clone, Default)]
pub enum PDFObject {
    #[default]
    Null,
    Bool(bool),
    Number(PDFNumber),
    String(PDFString),
    Name(PDFName),
    Arrray(PDFArray),
    Dictionary(PDFDictionary),
    Stream(PDFStream),
    Indirect(PDFIndirect),
}

impl PDFObject {
    pub fn get_value(&self, key: &str) -> Option<&PDFObject> {
        let name = PDFName::new(key);
        match self {
            PDFObject::Dictionary(d) => d.get(&name),
            PDFObject::Stream(s) => s.attribute(key),
            _ => None,
        }
    }

    pub fn get_value_as_string(&self, key: &str) -> Option<PDFResult<String>> {
        self.get_value(key).map(|obj| obj.as_string())
    }

    pub fn get_value_as_i64(&self, key: &str) -> Option<PDFResult<i64>> {
        self.get_value(key).map(|obj| obj.as_i64())
    }

    pub fn get_value_as_f64(&self, key: &str) -> Option<PDFResult<f64>> {
        self.get_value(key).map(|obj| obj.as_f64())
    }

    pub fn as_string(&self) -> PDFResult<String> {
        match self {
            PDFObject::Name(ref s) => Ok(s.to_string()),
            PDFObject::String(ref s) => Ok(s.to_string()),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} can't conveto to string",
                self
            ))),
        }
    }

    pub fn bytes(&self) -> PDFResult<Vec<u8>> {
        match self {
            PDFObject::String(PDFString::Literial(v)) => Ok(v.to_owned()),
            PDFObject::String(PDFString::HexString(v)) => Ok(v.to_owned()),
            PDFObject::Stream(s) => Ok(s.bytes()),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} convert to Bytes",
                self
            ))),
        }
    }

    pub fn as_array(&self) -> PDFResult<&PDFArray> {
        match self {
            PDFObject::Arrray(v) => Ok(v),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} can not convert to array",
                self
            ))),
        }
    }

    pub fn as_i64(&self) -> PDFResult<i64> {
        match self {
            PDFObject::Number(v) => Ok(v.as_i64()),
            _ => Err(PDFError::ObjectConvertFailure(
                "can't convert to number".to_string(),
            )),
        }
    }

    pub fn as_f64(&self) -> PDFResult<f64> {
        match self {
            PDFObject::Number(v) => Ok(v.as_f64()),
            _ => Err(PDFError::ObjectConvertFailure(
                "can't convert to number".to_string(),
            )),
        }
    }
}

// TODO: use marcro replace this

impl TryFrom<PDFObject> for PDFString {
    type Error = PDFError;
    fn try_from(value: PDFObject) -> Result<Self, Self::Error> {
        match value {
            PDFObject::String(r) => Ok(r),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} not PDFString ",
                value
            ))),
        }
    }
}
impl TryFrom<PDFObject> for PDFDictionary {
    type Error = PDFError;
    fn try_from(value: PDFObject) -> Result<Self, Self::Error> {
        match value {
            PDFObject::Dictionary(dict) => Ok(dict),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} not PDFDictionary",
                value
            ))),
        }
    }
}

impl TryFrom<PDFObject> for PDFArray {
    type Error = PDFError;
    fn try_from(value: PDFObject) -> Result<Self, Self::Error> {
        match value {
            PDFObject::Arrray(array) => Ok(array),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} not PDFArray",
                value
            ))),
        }
    }
}

impl TryFrom<PDFObject> for PDFName {
    type Error = PDFError;
    fn try_from(value: PDFObject) -> Result<Self, Self::Error> {
        match value {
            PDFObject::Name(r) => Ok(r),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} not PDFName",
                value
            ))),
        }
    }
}

impl TryFrom<PDFObject> for PDFStream {
    type Error = PDFError;
    fn try_from(value: PDFObject) -> Result<Self, Self::Error> {
        match value {
            PDFObject::Stream(r) => Ok(r),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} not PDFSTream",
                value
            ))),
        }
    }
}

impl TryFrom<PDFObject> for PDFIndirect {
    type Error = PDFError;
    fn try_from(value: PDFObject) -> Result<Self, Self::Error> {
        match value {
            PDFObject::Indirect(i) => Ok(i),
            _ => Err(PDFError::ObjectConvertFailure(format!(
                "{:?} not PDFIndirect",
                value
            ))),
        }
    }
}

//impl ToString for PDFObject {
//    fn to_string(&self) -> String {
//        match self {
//            PDFObject::Null => "NULL".to_string(),
//            PDFObject::Bool(b) => b.to_string(),
//            PDFObject::Name(name) => name.to_string(),
//            PDFObject::String(s) => s.to_string(),
//            _ => format!("{:?}", self),
//        }
//    }
//}
