use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

pub mod exponential;
pub mod postscript;
pub mod simple;
pub mod stiching;

#[derive(Debug, Clone)]
pub enum FunctionType {
    Simple,
    Exponential,
    Stitching,
    PostScript,
}

#[derive(Debug, Clone)]
pub struct CommonFunction {
    function_type: FunctionType,
    domain: Vec<f32>,
    range: Option<Vec<f32>>,
}

impl CommonFunction {
    pub fn try_new(obj: &PDFObject) -> PDFResult<Self> {
        let t = obj
            .get_value_as_u8("FunctionType")
            .ok_or(PDFError::FunctionError(format!(
                "FunctionType not in function:{:?}",
                obj
            )))??;

        let function_type = match t {
            0 => FunctionType::Simple,
            2 => FunctionType::Exponential,
            3 => FunctionType::Stitching,
            4 => FunctionType::PostScript,
            _ => {
                return Err(PDFError::FunctionError(format!(
                    "Invalid functiontype :{:?}",
                    t
                )));
            }
        };

        let domain = obj
            .get_value("Domain")
            .ok_or(PDFError::FunctionError(format!(
                "Domain is required in function:{:?}",
                obj
            )))?;
        let domain = domain.as_array()?;
        let domain = domain.iter().map(|x| x.as_f32().unwrap()).collect();
        let mut common = CommonFunction {
            function_type,
            domain,
            range: None,
        };
        match obj.get_value("Range") {
            Some(r) => {
                let r = r.as_array()?;
                let range = r.iter().map(|x| x.as_f32().unwrap()).collect();
                common.range = Some(range);
            }
            None => {
                if matches!(
                    common.function_type,
                    FunctionType::Simple | FunctionType::PostScript
                ) {
                    return Err(PDFError::FunctionError(format!(
                        "Range is need for FunctionType 0 or 4:{:?}",
                        obj
                    )));
                }
            }
        }
        Ok(common)
    }

    pub fn function_type(&self) -> &FunctionType {
        &self.function_type
    }

    pub fn domain(&self) -> &[f32] {
        self.domain.as_slice()
    }

    pub fn range(&self) -> Option<&[f32]> {
        self.range.as_deref()
    }
}
