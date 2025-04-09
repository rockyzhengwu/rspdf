use crate::{
    error::{PdfError, Result},
    object::PdfObject,
    xref::Xref,
};

#[derive(Debug, Clone, Default)]
pub struct PostScriptFunction {
    domain: Vec<f32>,
    range: Vec<f32>,
}

impl PostScriptFunction {
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let mut function = PostScriptFunction::default();

        let domain = obj
            .get_from_dict("Domain")
            .ok_or(PdfError::Function(
                "Type4 Function Domain is None".to_string(),
            ))?
            .as_array()
            .map_err(|_| PdfError::Function("Type4 function domain is not an array".to_string()))?;
        for v in domain.iter() {
            let value = v
                .as_number()
                .map_err(|_| PdfError::Function("Type4 Domain element is not number".to_string()))?
                .real();
            function.domain.push(value);
        }

        let range = obj
            .get_from_dict("Range")
            .ok_or(PdfError::Function("Type4 Range is None".to_string()))?
            .as_array()
            .map_err(|_| PdfError::Function("Type3 Function Range is not an array".to_string()))?;
        for v in range.iter() {
            let value = v
                .as_number()
                .map_err(|_| {
                    PdfError::Function("Type4 Function range element is not an number".to_string())
                })?
                .real();
            function.range.push(value);
        }
        let data = obj.as_stream()?.decode_data(Some(xref))?;

        Ok(function)
    }

    pub fn eval(&self, _inputs: &[f32]) -> Result<Vec<f32>> {
        let mut _stack: Vec<f32> = Vec::with_capacity(100);
        unimplemented!()
    }
}
