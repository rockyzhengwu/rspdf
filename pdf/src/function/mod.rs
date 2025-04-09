use exponential::Exponential;
use postscript::PostScriptFunction;
use sampled::Sampled;
use stitching::Stitching;

use crate::{
    error::{PdfError, Result},
    object::PdfObject,
    xref::Xref,
};

pub mod exponential;
pub mod postscript;
pub mod sampled;
pub mod stitching;

#[derive(Debug, Clone)]
pub enum Function {
    Type0(Sampled),
    Type2(Exponential),
    Type3(Stitching),
    Type4(PostScriptFunction),
}

pub fn create_function(obj: &PdfObject, xref: &Xref) -> Result<Function> {
    let t = obj
        .get_from_dict("FunctionType")
        .ok_or(PdfError::Function("FunctionType is None".to_string()))?
        .as_number()
        .map_err(|_| PdfError::Function("FunctionType need to be a number".to_string()))?;
    let ti = t.integer();
    match ti {
        0 => {
            let s = Sampled::try_new(obj, xref)?;
            Ok(Function::Type0(s))
        }
        2 => {
            let function = Exponential::try_new(obj, xref)?;
            Ok(Function::Type2(function))
        }
        3 => {
            let function = Stitching::try_new(obj, xref)?;
            Ok(Function::Type3(function))
        }
        4 => {
            let function = PostScriptFunction::try_new(obj, xref)?;
            Ok(Function::Type4(function))
        }
        _ => Err(PdfError::Function(format!(
            "FunctionType must be in [0,2,3,4] got:{:?}",
            ti
        ))),
    }
}

impl Function {
    pub fn eval(&self, inputs: &[f32]) -> Result<Vec<f32>> {
        match self {
            Function::Type0(f) => f.eval(inputs),
            Function::Type2(f) => f.eval(inputs),
            Function::Type3(f) => f.eval(inputs),
            Function::Type4(f) => f.eval(inputs),
        }
    }
}
