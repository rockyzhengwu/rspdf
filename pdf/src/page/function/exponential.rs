use crate::errors::PDFResult;
use crate::object::PDFObject;
use crate::page::function::common::CommonFunction;

#[derive(Debug, Clone)]
pub struct ExponentialFunction {
    common: CommonFunction,
    n: u32,
    c0: Vec<f32>,
    c1: Vec<f32>,
}

impl ExponentialFunction {
    pub fn try_new(obj: &PDFObject, common: CommonFunction) -> PDFResult<Self> {
        // TODO
        let c0 = vec![0.0];
        let c1 = vec![1.0];
        Ok(ExponentialFunction {
            common,
            c0,
            c1,
            n: 0,
        })
    }
}
