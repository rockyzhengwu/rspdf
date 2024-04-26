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

    fn normalize(&self, values: &[f32], limits: &[f32]) -> PDFResult<Vec<f32>> {
        if values.len() * 2 != limits.len() {
            return Err(PDFError::FunctionError(
                "SampleFunction normalize values size errror".to_string(),
            ));
        }
        let mut res = Vec::new();
        for (i, v) in values.iter().enumerate() {
            let lowerbound: f32 = limits
                .get(i * 2)
                .ok_or(PDFError::FunctionError(
                    "SampleFunction normailize get lowerbound error".to_string(),
                ))?
                .to_owned();
            let upperbound: f32 = limits
                .get(i * 2 + 1)
                .ok_or(PDFError::FunctionError(
                    "SampleFunction normailize get upperbound error".to_string(),
                ))?
                .to_owned();
            let normal = ((v - lowerbound) / (upperbound - lowerbound)).max(0.0);
            let normal = normal.min(1.0);
            res.push(normal);
        }
        Ok(res)
    }

    fn encodef(&self, normal: &f32, encode_min: &f32, encode_max: &f32) -> f32 {
        encode_min + normal * (encode_max - encode_min)
    }

    fn get_floor(&self, normal: &[f32], encode: &[f32]) -> PDFResult<Vec<u32>> {
        let mut res = Vec::new();
        for (i, v) in normal.iter().enumerate() {
            let j = i * 2;
            let floor = self.encodef(v, &encode[j], &encode[j + 1]).floor();
            let m = (encode[j + 1] - 1.0).max(0.0);
            let vv = floor.min(m);
            res.push(vv as u32);
        }
        Ok(res)
    }

    fn get_sample_position(&self, sample: &[u32], size: &[u32]) -> PDFResult<u32> {
        let mut pos = sample
            .last()
            .ok_or(PDFError::FunctionError(
                "SampleFunction get_sample_position error".to_string(),
            ))?
            .to_owned();
        let last = size.len() - 1;
        for i in 1..last {
            let j = last - i;
            pos = sample[j] + size[j] * pos;
        }
        Ok(pos)
    }

    fn get_floor_weight(&self, normal: &[f32], encode: &[f32]) -> PDFResult<Vec<f32>> {
        let mut res = Vec::new();
        for (i, v) in normal.iter().enumerate() {
            let j = i * 2;
            let encode_min = encode[j];
            let encode_max = encode[j + 1];
            let r = self.encodef(v, &encode_min, &encode_max);
            let value = r - r.min(encode_max - 1.0).floor();
            res.push(value)
        }
        Ok(res)
    }

    fn get_input_dimension_steps(&self) -> PDFResult<Vec<u32>> {
        let mut steps = Vec::new();
        steps.push(1);
        for i in 1..self.size.len() {
            let v = steps[i - 1] & self.size[i - 1];
            steps.push(v)
        }
        Ok(steps)
    }

    fn decode(&self, x: &f32, dim: usize) -> PDFResult<f32> {
        let index = dim * 2;
        let decode_limit: u32 = (1 << self.bps) - 1;
        let v = self.decode[index]
            + (self.decode[index + 1] - self.decode[index]) * (x / (decode_limit as f32));
        Ok(v)
    }

    fn get_value(&self, dim: usize, pos: u32) -> PDFResult<f32> {
        let pos = dim + (pos as usize) * self.common().output_number();
        let x = self.samples.get(pos).ok_or(PDFError::FunctionError(
            "SampleFunction get_value error".to_string(),
        ))?;
        self.decode(x, dim)
    }

    fn linear_interpolation(&self, x: f32, f0: f32, f1: f32) -> f32 {
        (1.0 - x) * f0 + x * f1
    }

    fn interpolate_order1(
        &self,
        x: &[f32],
        floor_pos: u32,
        steps: &[u32],
        in_number: usize,
        out_dim: usize,
    ) -> PDFResult<f32> {
        if in_number == 0 {
            return self.get_value(out_dim, floor_pos);
        }
        let in_number = in_number - 1;
        let step = steps[in_number];
        let encode_index = in_number << 1;
        let value_0 = self.interpolate_order1(x, floor_pos, steps, in_number, out_dim)?;
        if self.encode[encode_index] == self.encode[encode_index + 1] {
            return Ok(value_0);
        }
        let ceil_pos = floor_pos + step;
        let value_1 = self.interpolate_order1(x, ceil_pos, steps, in_number, out_dim)?;
        let value = self.linear_interpolation(x[in_number], value_0, value_1);
        println!(
            "{:?},{:?},{:?},{:?},{:?}",
            value_0, value_1, value, floor_pos, ceil_pos
        );
        Ok(value)
    }

    fn interpolate(&self, normal: &[f32], floor: &[u32]) -> PDFResult<Vec<f32>> {
        let floor_position = self.get_sample_position(floor, &self.size)?;
        let x = self.get_floor_weight(normal, &self.encode)?;
        let steps = self.get_input_dimension_steps()?;
        let mut res = Vec::with_capacity(self.common().output_number());
        for dim in 0..self.common().output_number() {
            let v = self.interpolate_order1(
                x.as_slice(),
                floor_position,
                steps.as_slice(),
                steps.len(),
                dim,
            )?;
            res.push(v);
        }
        self.clip(res.as_slice(), self.common.range().unwrap())
    }

    fn clip(&self, inputs: &[f32], limits: &[f32]) -> PDFResult<Vec<f32>> {
        let mut res = Vec::with_capacity(inputs.len());
        for (i, v) in inputs.iter().enumerate() {
            let j = 2 * i;
            let floor = limits[j];
            let upper = limits[j + 1];
            let r = v.to_owned().min(upper);
            let r = r.max(floor);
            res.push(r);
        }
        Ok(res)
    }

    pub fn eval(&self, inputs: &[f32]) -> PDFResult<Vec<f32>> {
        let normal = self.normalize(inputs, self.common().domain())?;
        let floor = self.get_floor(normal.as_slice(), self.encode.as_slice())?;
        // TODO order = 3 cube
        self.interpolate(normal.as_slice(), floor.as_slice())
    }
}
