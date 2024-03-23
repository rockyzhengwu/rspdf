use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use crate::page::function::CommonFunction;

pub struct SimpleFunction {
    common: CommonFunction,
    size: Vec<u32>,
    bits_per_sample: u8,
    order: Option<u8>,
    encode: Vec<f32>,
    decode: Vec<f32>,
    simples: Vec<f32>,
}

impl SimpleFunction {
    pub fn try_new(stream: &PDFObject) -> PDFResult<Self> {
        let common = CommonFunction::try_new(stream)?;
        let size = stream
            .get_value("Size")
            .ok_or(PDFError::FunctionError(format!(
                "Size is need in SimpleFunction :{:?}",
                stream
            )))?
            .as_array()?;
        let size: Vec<u32> = size.iter().map(|x| x.as_u32().unwrap()).collect();
        let bits_per_sample =
            stream
                .get_value_as_u8("BitsPerSample")
                .ok_or(PDFError::FunctionError(format!(
                    "BitsPerSample is need in SimpleFunction{:?}",
                    stream,
                )))??;
        let mut simple_function = SimpleFunction {
            common,
            size,
            bits_per_sample,
            order: None,
            encode: Vec::new(),
            decode: Vec::new(),
            simples: Vec::new(),
        };
        if let Some(Ok(o)) = stream.get_value_as_u8("Order") {
            simple_function.order = Some(o);
        }
        if let Some(enc) = stream.get_value("Encode") {
            let enc = enc.as_array()?;
            simple_function.encode = enc.iter().map(|x| x.as_f32().unwrap()).collect()
        }
        if let Some(enc) = stream.get_value("Decode") {
            let enc = enc.as_array()?;
            simple_function.decode = enc.iter().map(|x| x.as_f32().unwrap()).collect()
        }

        Ok(simple_function)
    }

    pub fn common(&self) -> &CommonFunction {
        &self.common
    }

    pub fn bits_per_sample(&self) -> u8 {
        self.bits_per_sample
    }

    pub fn size(&self) -> &[u32] {
        self.size.as_slice()
    }
}
