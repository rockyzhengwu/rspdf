use crate::{
    error::{PdfError, Result},
    object::{array::PdfArray, PdfObject},
    xref::Xref,
};

use super::value::{ColorRgb, ColorValue};
use super::{parse_colorspace, ColorSpace};

#[derive(Debug, Clone)]
pub struct Indexed {
    base: Box<ColorSpace>,
    hival: u8,
    lookup: Vec<u8>,
}

impl Indexed {
    pub fn default_value(&self) -> ColorValue {
        self.base.default_value()
    }

    pub fn try_new(arr: &PdfArray, xref: &Xref) -> Result<Self> {
        let base = arr
            .get(1)
            .ok_or(PdfError::Color("Indexed Base is None".to_string()))?;
        let base = parse_colorspace(base, xref)?;
        let hival = arr
            .get(2)
            .ok_or(PdfError::Color("Indexed Color hival is None".to_string()))?
            .as_number()
            .map_err(|_| PdfError::Color("Indexed Color hival is not a as_number".to_string()))?
            .integer() as u8;
        let lookup_obj = arr
            .get(3)
            .ok_or(PdfError::Color("Indexed color lookup is None".to_string()))?;
        let lookup = xref.read_object(lookup_obj)?;
        match lookup {
            PdfObject::Stream(stream) => {
                let color = Indexed {
                    base: Box::new(base),
                    hival,
                    lookup: stream.decode_data(Some(xref))?,
                };
                Ok(color)
            }
            PdfObject::LiteralString(s) => {
                let color = Indexed {
                    base: Box::new(base),
                    hival,
                    lookup: s.bytes().to_vec(),
                };
                Ok(color)
            }
            _ => Err(PdfError::Color(format!(
                "Indexed lookup need an stream or bytes string got :{:?}",
                lookup
            ))),
        }
    }

    pub fn rgb(&self, value: &ColorValue) -> Result<ColorRgb> {
        let v = value.values()[0];
        let pos = (v.ceil() as usize).max(0).min(self.hival as usize);
        let pos = pos * self.base.number_of_components();
        let mut cvs = Vec::new();
        for i in 0..self.base.number_of_components() {
            let n = self.lookup[pos + i] as f32;
            cvs.push(n / 255.0);
        }
        let value = ColorValue::new(cvs);
        self.base.rgb(&value)
    }

    pub fn number_of_components(&self) -> usize {
        1
    }
}
