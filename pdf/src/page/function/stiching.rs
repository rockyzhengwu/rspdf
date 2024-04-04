use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use crate::page::function::common::CommonFunction;

#[derive(Debug, Clone)]
pub struct StichingFunction {
    common: CommonFunction,
}

impl StichingFunction {
    pub fn try_new(obj: &PDFObject, common: CommonFunction) -> PDFResult<Self> {
        // TODO
        Ok(StichingFunction { common })
    }

    pub fn eval(&self, inputs: &[f32]) -> PDFResult<Vec<f32>> {
        unimplemented!()
    }
}
