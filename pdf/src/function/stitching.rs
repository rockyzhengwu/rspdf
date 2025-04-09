use crate::{
    document::Document,
    error::{PdfError, Result},
    function::create_function,
    object::PdfObject,
    xref::Xref,
};

use super::Function;

#[derive(Debug, Clone, Default)]
pub struct Stitching {
    domain: Vec<f32>,
    functions: Vec<Function>,
    bounds: Vec<f32>,
    encode: Vec<f32>,
    m: usize,
    n: usize,
}

impl Stitching {
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let mut function = Stitching::default();

        let domain = obj
            .get_from_dict("Domain")
            .ok_or(PdfError::Function(
                "Type3 Function Domain is None".to_string(),
            ))?
            .as_array()
            .map_err(|_| PdfError::Function("Type3 function domain is not an array".to_string()))?;
        for v in domain.iter() {
            let value = v
                .as_number()
                .map_err(|_| PdfError::Function("Type3 Domain element is not number".to_string()))?
                .real();
            function.domain.push(value);
        }
        function.m = function.domain.len() / 2;
        if function.m != 1 {
            return Err(PdfError::Function(
                "Type3 function with more than one input".to_string(),
            ))?;
        }
        let functions = obj
            .get_from_dict("Functions")
            .ok_or(PdfError::Function(
                "Type3 Function Functions is required".to_string(),
            ))?
            .as_array()
            .map_err(|_| PdfError::Function("Type3 Functions is not array".to_string()))?;
        for f in functions.iter() {
            let ff = create_function(f, xref)?;
            function.functions.push(ff);
        }
        if let Some(encode) = obj.get_from_dict("Encode") {
            let enc_array = encode.as_array().map_err(|_| {
                PdfError::Function("Type3 function Encode is not array".to_string())
            })?;
            for v in enc_array.iter() {
                let vv = v
                    .as_number()
                    .map_err(|_| {
                        PdfError::Function(
                            "Type3 Function Encode element is not number".to_string(),
                        )
                    })?
                    .real();
                function.encode.push(vv);
            }
        } else {
            return Err(PdfError::Function(
                "Type3 function Encode is rquired".to_string(),
            ));
        }
        Ok(function)
    }

    pub fn eval(&self, inputs: &[f32]) -> Result<Vec<f32>> {
        unimplemented!()
    }
}
