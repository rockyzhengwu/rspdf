use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

#[derive(Debug, Default, Clone)]
pub struct IccProfile {}

#[derive(Debug, Clone)]
pub struct IccBased {
    n: u8,
    profile: IccProfile,
}

impl IccBased {
    pub fn try_new(obj: &PDFObject) -> PDFResult<Self> {
        let n = obj
            .get_value_as_u8("N")
            .ok_or(PDFError::ColorError("IccBased need N".to_string()))??;
        // TODO parse ICCProfile
        Ok(IccBased {
            n,
            profile: IccProfile::default(),
        })
    }
}
