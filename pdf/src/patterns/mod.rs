use shading::ShadingPattern;
use tiling::TilingPattern;

use crate::{
    error::{PdfError, Result},
    object::PdfObject,
};

pub mod shading;
pub mod tiling;

pub enum Pattern {
    Shading(ShadingPattern),
    Tiling(TilingPattern),
}

impl Pattern {
    pub fn try_new(obj: &PdfObject) -> Result<Self> {
        let pt = obj
            .get_from_dict("PatternType")
            .ok_or(PdfError::Path("PatternType is None".to_string()))?
            .as_number()
            .map_err(|_| PdfError::Pattern("Pattern PatternType is not as_number".to_string()))?
            .integer();
        match pt {
            1 => {
                let tiling = TilingPattern::try_new(obj)?;
                return Ok(Pattern::Tiling(tiling));
            }
            2 => {
                let shading = ShadingPattern::try_new(obj)?;
                return Ok(Pattern::Shading(shading));
            }
            _ => {
                return Err(PdfError::Pattern("PatternType mustbe 1 or 2".to_string()));
            }
        }
    }
}
