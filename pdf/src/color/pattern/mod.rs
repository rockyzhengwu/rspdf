use super::value::ColorValue;
use crate::error::{PdfError, Result};
use crate::object::PdfObject;
use crate::xref::Xref;

pub mod tiling;
use tiling::TilingPattern;

#[derive(Debug, Clone, Default)]
pub struct PatternColorSpace {}

impl PatternColorSpace {
    pub fn try_new(obj: &PdfObject) -> Result<Self> {
        unimplemented!()
    }

    pub fn default_value(&self) -> ColorValue {
        ColorValue::default()
    }
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Tiling(TilingPattern),
    Shading,
}

impl Pattern {
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let pt = obj
            .get_from_dict("PatternType")
            .ok_or(PdfError::Color("Pattern has no Pattern type ".to_string()))?;
        let pt = pt
            .as_number()
            .map_err(|_| PdfError::Color("Color error".to_string()))?
            .integer();
        println!("{:?}", pt);
        match pt {
            1 => {
                let p = TilingPattern::try_new(obj, xref)?;
                return Ok(Pattern::Tiling(p));
            }
            _ => {
                unimplemented!("Shading pattern not implement")
            }
        }
    }
}
