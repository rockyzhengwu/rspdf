use crate::error::{PdfError, Result};

use crate::filter::apply_filter;
use crate::object::dictionary::PdfDict;
use crate::object::PdfObject;
use crate::xref::Xref;

#[derive(Debug, PartialEq, Clone)]
pub struct PdfStream {
    dict: PdfDict,
    data: Vec<u8>,
}

impl PdfStream {
    pub fn new(dict: PdfDict, data: Vec<u8>) -> Self {
        Self { dict, data }
    }

    pub fn get_from_dict(&self, key: &str) -> Option<&PdfObject> {
        self.dict.get(key)
    }

    pub fn dict(&self) -> &PdfDict {
        &self.dict
    }
    pub fn raw_data(&self) -> &[u8] {
        self.data.as_slice()
    }
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data
    }

    pub fn decode_data(&self, xref: Option<&Xref>) -> Result<Vec<u8>> {
        let params = match self.dict.get("DecodeParms") {
            Some(obj) => match xref {
                Some(d) => Some(d.read_object(obj)?),
                None => Some(obj.to_owned()),
            },
            None => None,
        };
        if let Some(filter) = self.dict.get("Filter") {
            match filter {
                PdfObject::Name(n) => match params {
                    Some(PdfObject::Dict(d)) => {
                        apply_filter(n.name(), self.data.as_slice(), Some(&d))
                    }
                    None => apply_filter(n.name(), self.data.as_slice(), None),
                    _ => Err(PdfError::Filter("Filter param must be Dict ".to_string())),
                },
                PdfObject::Array(filters) => match params {
                    Some(PdfObject::Array(param_arr)) => {
                        assert_eq!(filters.len(), param_arr.len());
                        let mut data = self.data.clone();
                        for (name, param) in filters.iter().zip(param_arr.iter()) {
                            data = apply_filter(
                                name.as_name()?.name(),
                                data.as_slice(),
                                Some(param.as_dict()?),
                            )?;
                        }
                        Ok(data)
                    }
                    None => {
                        let mut data = self.data.clone();
                        for name in filters.iter() {
                            data = apply_filter(name.as_name()?.name(), data.as_slice(), None)?;
                        }
                        Ok(data)
                    }
                    _ => Err(PdfError::Filter("Filter param must be Dict ".to_string())),
                },
                _ => Err(PdfError::Filter(
                    "Stream filter must be PdfName or Array".to_string(),
                )),
            }
        } else {
            Ok(self.data.clone())
        }
    }
}
