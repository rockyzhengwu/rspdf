use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use crate::page::function::common::CommonFunction;
use crate::page::function::exponential::ExponentialFunction;
use crate::page::function::postscript::PostScriptFunction;
use crate::page::function::sample::SampleFunction;
use crate::page::function::stiching::StichingFunction;

pub mod common;
pub mod exponential;
pub mod postscript;
pub mod sample;
pub mod stiching;

#[derive(Debug, Clone)]
pub enum PDFFunction {
    Simple(sample::SampleFunction),
    Exponential(exponential::ExponentialFunction),
    Stitching(stiching::StichingFunction),
    PostScript(postscript::PostScriptFunction),
}

impl PDFFunction {
    pub fn try_new(obj: &PDFObject) -> PDFResult<Self> {
        let t = obj
            .get_value_as_u8("FunctionType")
            .ok_or(PDFError::FunctionError(format!(
                "FunctionType not in function:{:?}",
                obj
            )))??;

        let domain = obj
            .get_value("Domain")
            .ok_or(PDFError::FunctionError(format!(
                "Domain is required in function:{:?}",
                obj
            )))?;
        let domain = domain.as_array()?;
        let domain = domain.iter().map(|x| x.as_f32().unwrap()).collect();

        let range: Option<Vec<f32>> = match obj.get_value("Range") {
            Some(r) => {
                let r = r.as_array()?;
                let range = r.iter().map(|x| x.as_f32().unwrap()).collect();
                Some(range)
            }
            None => {
                if matches!(t, 0 | 4) {
                    return Err(PDFError::FunctionError(format!(
                        "Range is need for FunctionType 0 or 4:{:?}",
                        obj
                    )));
                }
                None
            }
        };
        let common = CommonFunction::new(domain, range);
        match t {
            0 => {
                let s = SampleFunction::try_new(obj, common)?;
                Ok(PDFFunction::Simple(s))
            }
            2 => {
                // PDFFunction::Exponential
                let f = ExponentialFunction::try_new(obj, common)?;
                Ok(PDFFunction::Exponential(f))
            }
            3 => {
                let f = StichingFunction::try_new(obj, common)?;
                Ok(PDFFunction::Stitching(f))
            }
            4 => {
                let f = PostScriptFunction::try_new(obj, common)?;
                Ok(PDFFunction::PostScript(f))
            }
            _ => Err(PDFError::FunctionError(format!(
                "Invalid functiontype :{:?}",
                t
            ))),
        }
    }
}
