use std::cmp;
use std::cmp::Ord;

use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use crate::page::function::common::CommonFunction;

#[derive(Debug, Clone)]
pub struct SampleFunction {
    common: CommonFunction,
    size: Vec<u32>,
    bits_per_sample: u8,
    order: Option<u8>,
    encode: Vec<f32>,
    decode: Vec<f32>,
    samples: Vec<u8>,
}

impl SampleFunction {
    pub fn try_new(stream: &PDFObject, common: CommonFunction) -> PDFResult<Self> {
        let size = stream
            .get_value("Size")
            .ok_or(PDFError::FunctionError(format!(
                "Size is need in SampleFunction :{:?}",
                stream
            )))?
            .as_array()?;
        let size: Vec<u32> = size.iter().map(|x| x.as_u32().unwrap()).collect();
        let bits_per_sample =
            stream
                .get_value_as_u8("BitsPerSample")
                .ok_or(PDFError::FunctionError(format!(
                    "BitsPerSample is need in SampleFunction{:?}",
                    stream,
                )))??;
        let mut sample_function = SampleFunction {
            common,
            size,
            bits_per_sample,
            order: None,
            encode: Vec::new(),
            decode: Vec::new(),
            samples: Vec::new(),
        };
        if let Some(Ok(o)) = stream.get_value_as_u8("Order") {
            sample_function.order = Some(o);
        }
        if let Some(enc) = stream.get_value("Encode") {
            let enc = enc.as_array()?;
            sample_function.encode = enc.iter().map(|x| x.as_f32().unwrap()).collect()
        }
        if let Some(enc) = stream.get_value("Decode") {
            let enc = enc.as_array()?;
            sample_function.decode = enc.iter().map(|x| x.as_f32().unwrap()).collect()
        }
        sample_function.samples = stream.bytes()?;
        println!(
            "{:?},{:?},{:?},{:?}, {:?}",
            sample_function.samples.len(),
            sample_function.common.output_number(),
            sample_function.common().input_number(),
            sample_function.bits_per_sample,
            sample_function.encode,
        );

        Ok(sample_function)
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
    fn interpolate(&self, x: f32, xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> f32 {
        ymin + ((x - xmin) * (ymax - ymin) / (xmax - xmin))
    }

    pub fn eval(&self, inputs: &[f32]) -> Vec<f32> {
        let mut output = Vec::new();
        for (i, v) in inputs.iter().enumerate() {
            let low = self.common().get_domain(i * 2).to_owned();
            let up = self.common().get_domain(i * 2 + 1).to_owned();
            let elow = self.encode.get(i * 2).unwrap().to_owned();
            let eup = self.encode.get(i * 2 + 1).unwrap().to_owned();
            let x = v.max(low).min(up);
            let x = self.interpolate(x, low, up, elow, eup);
            let size = self.size.get(i).unwrap().to_owned() as f32;
            let x = x.max(0.0).min(size);

        }
        output
    }
}
