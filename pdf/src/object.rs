use std::collections::HashMap;
use std::fmt::{self, Display};

use crate::errors::{PDFError, PDFResult};
use crate::filter::asciihex_decode;
use crate::filter::decode;

#[derive(Clone, Debug)]
pub struct PDFIndirect {
    number: u32,
    gen: u16,
}

impl PDFIndirect {
    pub fn new(number: u32, gen: u16) -> Self {
        PDFIndirect { number, gen }
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn gen(&self) -> u16 {
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

impl Display for PDFName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub type PDFDictionary = HashMap<String, PDFObject>;

#[derive(Clone)]
pub enum PDFString {
    HexString(Vec<u8>),
    Literial(Vec<u8>),
}

impl fmt::Debug for PDFString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PDFString::HexString(s) => {
                write!(f, "HexString({:?})", String::from_utf8_lossy(s))
            }
            PDFString::Literial(s) => {
                write!(f, "LiteralString({:?})", String::from_utf8_lossy(s))
            }
        }
    }
}

impl PDFString {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::HexString(v) => v.as_slice(),
            Self::Literial(v) => v.as_slice(),
        }
    }

    // decode hex to binary
    pub fn binary_bytes(&self) -> PDFResult<Vec<u8>> {
        match self {
            Self::HexString(bytes) => asciihex_decode::asciihex_decode(bytes),
            Self::Literial(v) => Ok(v.to_owned()),
        }
    }
}

impl Display for PDFString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PDFString::HexString(hex) => {
                write!(f, "{}", String::from_utf8_lossy(hex.as_slice()))
            }
            PDFString::Literial(literial) => {
                write!(f, "{}", String::from_utf8_lossy(literial.as_slice()))
            }
        }
    }
}

#[derive(Clone)]
pub struct PDFStream {
    offset: u64,
    dict: PDFDictionary,
    buffer: Vec<u8>,
}

impl fmt::Debug for PDFStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PDFStream(offset:{}, dict:{:?}, buffer:{:?})",
            self.offset,
            self.dict,
            String::from_utf8_lossy(self.buffer.as_slice())
        )
    }
}

impl PDFStream {
    pub fn new(offset: u64, dict: PDFDictionary) -> Self {
        PDFStream {
            offset,
            dict,
            buffer: Vec::new(),
        }
    }

    pub fn set_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer = buffer;
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn length(&self) -> Option<&PDFObject> {
        self.dict.get("Length")
    }

    pub fn attribute(&self, name: &str) -> Option<&PDFObject> {
        self.dict.get(name)
    }

    pub fn dict(&self) -> PDFDictionary {
        self.dict.clone()
    }

    pub fn bytes(&self) -> Vec<u8> {
        let filter = self.attribute("Filter");
        let decode_params = self.attribute("DecodeParms");

        let mut filters = Vec::new();
        let mut params: Vec<Option<&PDFObject>> = Vec::new();
        match filter {
            Some(PDFObject::Name(name)) => {
                filters.push(name.to_string());
                params.push(decode_params.to_owned());
            }
            Some(PDFObject::Arrray(arr)) => {
                for (i, a) in arr.iter().enumerate() {
                    filters.push(a.as_string().unwrap());
                    if let Some(PDFObject::Arrray(arr)) = decode_params {
                        params.push(arr.get(i))
                    } else {
                        params.push(None);
                    }
                }
            }
            _ => {}
        }
        let mut buffer = self.buffer.to_owned();
        for (i, fname) in filters.iter().enumerate() {
            let param = params.get(i).unwrap().to_owned();
            buffer = decode(fname, buffer.as_slice(), param).unwrap();
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
    pub fn as_u64(&self) -> u64 {
        match *self {
            PDFNumber::Real(r) => r as u64,
            PDFNumber::Integer(i) => i as u64,
        }
    }

    pub fn as_u32(&self) -> u32 {
        match *self {
            PDFNumber::Real(r) => r as u32,
            PDFNumber::Integer(i) => i as u32,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match *self {
            PDFNumber::Real(r) => r as u8,
            PDFNumber::Integer(i) => i as u8,
        }
    }

    pub fn as_i32(&self) -> i32 {
        match *self {
            PDFNumber::Real(r) => r as i32,
            PDFNumber::Integer(i) => i as i32,
        }
    }

    pub fn as_f32(&self) -> f32 {
        match *self {
            PDFNumber::Real(r) => r as f32,
            PDFNumber::Integer(i) => i as f32,
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
        match self {
            PDFObject::Dictionary(d) => d.get(key),
            PDFObject::Stream(s) => s.attribute(key),
            _ => None,
        }
    }
    pub fn set_value(&mut self, key: &str, obj: PDFObject) -> PDFResult<()> {
        match self {
            PDFObject::Dictionary(d) => d.insert(key.to_string(), obj),
            PDFObject::Stream(s) => s.dict.insert(key.to_string(), obj),
            _ => {
                return Err(PDFError::ObjectConvertFailure(
                    "Only stream and dict has set".to_string(),
                ));
            }
        };
        Ok(())
    }

    pub fn has_dict(&self) -> bool {
        matches!(self, PDFObject::Dictionary(_) | PDFObject::Stream(_))
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

    pub fn get_value_as_i32(&self, key: &str) -> Option<PDFResult<i32>> {
        self.get_value(key).map(|obj| obj.as_i32())
    }

    pub fn get_value_as_u32(&self, key: &str) -> Option<PDFResult<u32>> {
        self.get_value(key).map(|obj| obj.as_u32())
    }
    pub fn get_value_as_u8(&self, key: &str) -> Option<PDFResult<u8>> {
        self.get_value(key).map(|obj| obj.as_u8())
    }

    pub fn get_value_as_f32(&self, key: &str) -> Option<PDFResult<f32>> {
        self.get_value(key).map(|obj| obj.as_f32())
    }

    pub fn as_string(&self) -> PDFResult<String> {
        match self {
            PDFObject::Name(ref s) => Ok(s.to_string()),
            PDFObject::String(ref s) => Ok(s.to_string()),
            PDFObject::Null => Ok("null".to_string()),
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
    pub fn as_u32(&self) -> PDFResult<u32> {
        match self {
            PDFObject::Number(v) => Ok(v.as_u32()),
            _ => Err(PDFError::ObjectConvertFailure(
                "can't convert to number".to_string(),
            )),
        }
    }

    pub fn as_u8(&self) -> PDFResult<u8> {
        match self {
            PDFObject::Number(v) => Ok(v.as_u8()),
            _ => Err(PDFError::ObjectConvertFailure(
                "can't convert to number".to_string(),
            )),
        }
    }
    pub fn as_u64(&self) -> PDFResult<u64> {
        match self {
            PDFObject::Number(v) => Ok(v.as_u64()),
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
    pub fn as_i32(&self) -> PDFResult<i32> {
        match self {
            PDFObject::Number(v) => Ok(v.as_i32()),
            _ => Err(PDFError::ObjectConvertFailure(
                "can't convert to number".to_string(),
            )),
        }
    }

    pub fn as_f32(&self) -> PDFResult<f32> {
        match self {
            PDFObject::Number(v) => Ok(v.as_f32()),
            _ => Err(PDFError::ObjectConvertFailure(
                "can't convert to number".to_string(),
            )),
        }
    }

    pub fn is_indirect(&self) -> bool {
        matches!(self, PDFObject::Indirect(_))
    }

    pub fn as_bool(&self) -> PDFResult<bool> {
        match self {
            PDFObject::Bool(b) => Ok(b.to_owned()),
            _ => Err(PDFError::ObjectConvertFailure(
                "can't convert to bool".to_string(),
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
