use std::io::Cursor;

use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use crate::page::function::common::CommonFunction;
use bitstream_io::{BigEndian, BitRead, BitReader};

#[derive(Debug, Clone)]
pub struct SampleFunction {
    common: CommonFunction,
    size: Vec<u32>,
    bps: u8,
    order: Option<u8>,
    encode: Vec<f32>,
    decode: Vec<f32>,
    samples: Vec<f32>,
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
        let bps = stream
            .get_value_as_u8("BitsPerSample")
            .ok_or(PDFError::FunctionError(format!(
                "BitsPerSample is need in SampleFunction{:?}",
                stream,
            )))??;
        let mut sample_function = SampleFunction {
            common,
            size,
            bps,
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
        let bytes = stream.bytes()?;
        let mut bitreader = BitReader::endian(Cursor::new(&bytes), BigEndian);

        let mut samples: Vec<f32> = Vec::new();
        while let Ok(v) = bitreader.read::<i32>(sample_function.bps as u32) {
            samples.push(v as f32);
        }
        let mut t = 1;
        let n = sample_function.common.output_number() as usize;
        for s in sample_function.size() {
            t *= s.to_owned() as usize * n;
        }
        if t != samples.len() {
            return Err(PDFError::FunctionError(
                "SampleFunction sampels number error".to_string(),
            ));
        }
        sample_function.samples = samples;
        Ok(sample_function)
    }

    pub fn common(&self) -> &CommonFunction {
        &self.common
    }

    pub fn bits_per_sample(&self) -> u8 {
        self.bps
    }

    pub fn size(&self) -> &[u32] {
        self.size.as_slice()
    }
    fn interpolate(&self, x: f32, xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> f32 {
        ymin + ((x - xmin) * (ymax - ymin) / (xmax - xmin))
    }

    pub fn eval(&self, inputs: &[f32]) -> PDFResult<Vec<f32>> {
        let mut output = Vec::new();
        let mut e = Vec::new();
        let mut prev = Vec::new();
        let mut next = Vec::new();
        let mut efrac: Vec<f32> = Vec::new();
        for (i, v) in inputs.iter().enumerate() {
            let low = self.common().get_domain(i * 2).to_owned();
            let up = self.common().get_domain(i * 2 + 1).to_owned();
            let elow = self.encode.get(i * 2).unwrap().to_owned();
            let eup = self.encode.get(i * 2 + 1).unwrap().to_owned();
            let x = v.max(low).min(up);
            let x = self.interpolate(x, low, up, elow, eup);
            let size = self.size.get(i).unwrap().to_owned() as f32;
            let x = x.max(0.0).min(size);
            prev.push(x.floor());
            next.push(x.ceil());
            efrac.push(x - x.floor());
            e.push(x);
        }
        let n = self.common.output_number();
        let m = self.common.input_number();
        for i in 0..n {
            match m {
                1 => {
                    let pos = prev.first().unwrap().to_owned() as usize * n + i;
                    let a = self.samples.get(pos).unwrap();
                    let pos = next.first().unwrap().to_owned() as usize * n + i;
                    let b = self.samples.get(pos).unwrap();
                    let ab = a + (b - a) * efrac.first().unwrap();
                    let o = self.interpolate(
                        ab,
                        0.0,
                        1.0,
                        self.decode.get(2 * i).unwrap().to_owned(),
                        self.decode.get(2 * i + 1).unwrap().to_owned(),
                    );
                    let rl = self.common().get_range(2 * i).to_owned();
                    let ru = self.common().get_range(2 * i + 1).to_owned();
                    let ov = o.max(rl).min(ru);
                    output.push(ov)
                }
                2 => {
                    // TODO
                }
                _ => {
                    // TODO
                }
            }
        }
        Ok(output)
    }
}
