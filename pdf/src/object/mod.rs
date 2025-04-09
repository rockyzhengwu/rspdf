use crate::error::{PdfError, Result};
use crate::object::array::PdfArray;
use crate::object::bool::PdfBool;
use crate::object::dictionary::PdfDict;
use crate::object::name::PdfName;
use crate::object::number::PdfNumber;
use crate::object::stream::PdfStream;
use crate::object::string::{PdfHexString, PdfLiteral};

pub mod array;
pub mod bool;
pub mod dictionary;
pub mod name;
pub mod number;
pub mod stream;
pub mod string;

pub type ObjectId = (u32, u16);

#[derive(Debug, PartialEq, Clone)]
pub enum PdfObject {
    Null,
    Bool(PdfBool),
    Name(PdfName),
    LiteralString(PdfLiteral),
    HexString(PdfHexString),
    Number(PdfNumber),
    Stream(PdfStream),
    Array(PdfArray),
    Dict(PdfDict),
    Indirect(ObjectId),
}

impl PdfObject {
    pub fn is_null(&self) -> bool {
        matches!(self, PdfObject::Null)
    }

    pub fn is_indirect(&self) -> bool {
        matches!(self, PdfObject::Indirect(_))
    }

    pub fn get_from_dict(&self, key: &str) -> Option<&PdfObject> {
        match self {
            PdfObject::Dict(dict) => dict.get(key),
            PdfObject::Stream(stream) => stream.get_from_dict(key),
            _ => None,
        }
    }

    pub fn integer(&self) -> Result<i32> {
        match self {
            PdfObject::Number(num) => Ok(num.integer()),
            _ => Err(PdfError::Object(format!(
                "PdfObject not a Number got:{:?}",
                self
            ))),
        }
    }

    pub fn as_number(&self) -> Result<&PdfNumber> {
        match self {
            PdfObject::Number(n) => Ok(n),
            _ => Err(PdfError::Object(format!(
                "PdfObject as Number need Number got:{:?}",
                self
            ))),
        }
    }

    pub fn as_dict(&self) -> Result<&PdfDict> {
        match self {
            PdfObject::Dict(d) => Ok(&d),
            _ => Err(PdfError::Object(format!(
                "PdfObject as dict need Dict or Stream got:{:?}",
                self
            ))),
        }
    }

    pub fn to_dict(self) -> Result<PdfDict> {
        match self {
            PdfObject::Dict(d) => Ok(d),
            _ => Err(PdfError::Object(format!(
                "PdfObject as dict need Dict or Stream got:{:?}",
                self
            ))),
        }
    }

    pub fn as_literal(&self) -> Result<&PdfLiteral> {
        match self {
            PdfObject::LiteralString(s) => Ok(s),
            _ => Err(PdfError::Object(format!(
                "PdfObject as LiteraString need LiteralString got:{:?}",
                self
            ))),
        }
    }
    pub fn as_hex_string(&self) -> Result<&PdfHexString> {
        match self {
            PdfObject::HexString(s) => Ok(s),
            _ => Err(PdfError::Object(format!(
                "PdfObject as HexString need HexString got:{:?}",
                self
            ))),
        }
    }

    pub fn as_name(&self) -> Result<&PdfName> {
        match self {
            PdfObject::Name(name) => Ok(name),
            _ => Err(PdfError::Object(format!(
                "PdfObject as name need Name got:{:?}",
                self
            ))),
        }
    }

    pub fn as_array(&self) -> Result<&PdfArray> {
        match self {
            PdfObject::Array(arr) => Ok(arr),
            _ => Err(PdfError::Object(format!(
                "PdfObject as array need PdfArray got:{:?}",
                self
            ))),
        }
    }

    pub fn as_stream(&self) -> Result<&PdfStream> {
        match self {
            PdfObject::Stream(s) => Ok(s),
            _ => Err(PdfError::Object(format!(
                "PdfObject as array need PdfStream got:{:?}",
                self
            ))),
        }
    }

    pub fn to_stream(self) -> Result<PdfStream> {
        match self {
            PdfObject::Stream(s) => Ok(s),
            _ => Err(PdfError::Object(format!(
                "PdfObject as array need PdfStream got:{:?}",
                self
            ))),
        }
    }

    pub fn as_bool(&self) -> Result<&PdfBool> {
        match self {
            PdfObject::Bool(b) => Ok(b),
            _ => Err(PdfError::Object(format!(
                "PdfObject as pdfbool need pdfbool got:{:?}",
                self
            ))),
        }
    }
}
