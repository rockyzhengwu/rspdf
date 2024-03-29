use crate::errors::PDFResult;
use crate::object::PDFObject;
use crate::page::function::common::CommonFunction;

#[derive(Debug, Clone)]
pub struct ExponentialFunction {
    common: CommonFunction,
}

impl ExponentialFunction {
    pub fn try_new(obj: &PDFObject, common: CommonFunction) -> PDFResult<Self> {
        // TODO
        Ok(ExponentialFunction { common })
    }
}
