use std::io::Cursor;

use bitstream_io::{BigEndian, BitRead, BitReader};

use crate::{
    error::{PdfError, Result},
    object::PdfObject,
    xref::Xref,
};

#[derive(Debug, Clone, Default)]
pub struct Sampled {
    domain: Vec<f32>,
    range: Vec<f32>,
    size: Vec<usize>,
    bits_per_sample: u8,
    order: u8,
    encode: Vec<f32>,
    decode: Vec<f32>,
    m: usize,
    n: usize,
    samples: Vec<f32>,
    idx_offset: Vec<usize>,
}

impl Sampled {
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let obj = obj
            .as_stream()
            .map_err(|_| PdfError::Function("Sampled need a stream ".to_string()))?;
        let mut function = Sampled::default();
        let domain = obj
            .get_from_dict("Domain")
            .ok_or(PdfError::Function(
                "Sample Function Domain is None".to_string(),
            ))?
            .as_array()
            .map_err(|_| {
                PdfError::Function("Sampled function domain is not an array".to_string())
            })?;
        for v in domain.iter() {
            let value = v
                .as_number()
                .map_err(|_| PdfError::Function("Domain element is not number".to_string()))?
                .real();
            function.domain.push(value);
        }
        function.m = function.domain.len() / 2;

        let range = obj
            .get_from_dict("Range")
            .ok_or(PdfError::Function("Sampled Range is None".to_string()))?
            .as_array()
            .map_err(|_| {
                PdfError::Function("Sampled Function Range is not an array".to_string())
            })?;
        for v in range.iter() {
            let value = v
                .as_number()
                .map_err(|_| {
                    PdfError::Function(
                        "Sampled Function range element is not an number".to_string(),
                    )
                })?
                .real();
            function.range.push(value);
        }
        function.n = range.len() / 2;
        let size_obj = obj
            .get_from_dict("Size")
            .ok_or(PdfError::Function(
                "Smapled Function Size param is None".to_string(),
            ))?
            .as_array()
            .map_err(|_| PdfError::Function("Smapled Function Size need an aray".to_string()))?;
        for v in size_obj.iter() {
            let value = v
                .as_number()
                .map_err(|_| {
                    PdfError::Function(
                        "Sampled Function Size Value need to be a number".to_string(),
                    )
                })?
                .integer() as usize;
            function.size.push(value);
        }

        let corner_size: usize = 1 << function.m;
        let mut idx_offset: Vec<usize> = Vec::with_capacity(corner_size);
        for i in 0..corner_size {
            let mut idx: usize = 0;
            let mut j = function.m - 1;
            let mut t = i;
            while j > 0 {
                let bit = if function.size[j] == 1 {
                    0
                } else {
                    (t >> (function.m - 1)) & 1
                };
                idx = (idx + bit) * (function.size[j - 1]);
                j -= 1;
                t <<= 1;
            }
            let bit = if function.size[0] == 1 {
                0
            } else {
                (t >> (function.m - 1)) & 1
            };
            idx_offset.push((idx + bit) * function.n);
        }
        function.idx_offset = idx_offset;

        let bits_per_smaple = obj
            .get_from_dict("BitsPerSample")
            .ok_or(PdfError::Color(
                "Sampled Function BitsPerSample is None".to_string(),
            ))?
            .as_number()
            .map_err(|_| {
                PdfError::Function("Sampled Function BitsPerSample is not number".to_string())
            })?
            .integer() as u8;
        if !matches!(bits_per_smaple, 1 | 2 | 4 | 8 | 12 | 16 | 24 | 32) {
            return Err(PdfError::Function(format!(
                "Sampled Function BitsPerSample is invalid got  {:?}",
                bits_per_smaple
            )));
        }
        function.bits_per_sample = bits_per_smaple;
        if let Some(o) = obj.get_from_dict("Order") {
            let ov = o.integer().map_err(|_| {
                PdfError::Function("Sampled function order is not number".to_string())
            })?;
            function.order = ov as u8;
        } else {
            function.order = 1_u8;
        }
        if let Some(encode) = obj.get_from_dict("Encode") {
            let enc_array = encode.as_array().map_err(|_| {
                PdfError::Function("Sampled function Encode is not array".to_string())
            })?;
            for v in enc_array.iter() {
                let vv = v
                    .as_number()
                    .map_err(|_| {
                        PdfError::Function(
                            "Sampled Function Encode element is not number".to_string(),
                        )
                    })?
                    .real();
                function.encode.push(vv);
            }
        } else {
            for s in function.size.iter() {
                function.encode.push(0.0);
                function.encode.push(s.to_owned() as f32);
            }
        }
        if let Some(decode) = obj.get_from_dict("Decode") {
            let dec_array = decode.as_array().map_err(|_| {
                PdfError::Function("Sampled function Decode is not array".to_string())
            })?;
            for v in dec_array.iter() {
                let vv = v
                    .as_number()
                    .map_err(|_| {
                        PdfError::Function(
                            "Sampled Function Decode element is not number".to_string(),
                        )
                    })?
                    .real();
                function.decode.push(vv);
            }
        } else {
            function.decode = function.range.clone();
        }
        let mut samples = Vec::new();
        let bytes = obj.decode_data(Some(xref))?;
        let mut bitreader = BitReader::endian(Cursor::new(bytes), BigEndian);
        let sample_mul = 1.0 / (2.0_f32.powf(function.bits_per_sample as f32) as f32 - 1.0);
        while let Ok(v) = bitreader.read::<i32>(function.bits_per_sample as u32) {
            samples.push((v as f32) * sample_mul);
        }
        function.samples = samples;
        Ok(function)
    }

    pub fn eval(&self, inputs: &[f32]) -> Result<Vec<f32>> {
        let mut e: Vec<usize> = Vec::with_capacity(self.m);
        let mut efrac0: Vec<f32> = Vec::with_capacity(self.m);
        let mut efrac1: Vec<f32> = Vec::with_capacity(self.m);

        let normal = normalize(inputs, self.domain.as_slice())?;
        for (i, v) in normal.iter().enumerate() {
            let mut x = interpolate(
                v,
                self.domain[2 * i],
                self.domain[2 * i + 1],
                self.encode[2 * i],
                self.encode[2 * i + 1],
            );
            let s = self.size[i] as f32;
            if x > s {
                x = s;
            }
            if x < 0.0 {
                x = 0.0;
            } else if x > (self.size[i] - 1) as f32 {
                x = (self.size[i] - 1) as f32;
            }
            let floor = x.floor() as usize;
            if floor == self.size[i] - 1 && self.size[i] > 1 {
                e.push(self.size[i] - 2);
            } else {
                e.push(floor);
            }
            efrac0.push(x - floor as f32);
            efrac1.push(1.0 - (x - floor as f32));
        }
        let mut idx0: usize = 0;
        let mut k = self.m - 1;
        while k > 0 {
            idx0 = (idx0 + e[k]) * self.size[k - 1];
            k -= 1;
        }
        idx0 = (idx0 + e[0]) * self.n;
        let mut out = Vec::new();
        for i in 0..self.n {
            let mut values: Vec<f32> = Vec::with_capacity(1 << self.m);
            for j in 0..(1 << self.m) {
                values.push(self.samples[idx0 + self.idx_offset[j] + i]);
            }
            let mut j = 0;
            let mut t = 1 << self.m;
            while j < self.m {
                let mut k = 0;
                while k < t {
                    values[k >> 1] = efrac0[j] * values[k] + efrac1[j] * values[k + 1];
                    k += 2;
                }
                j += 1;
                t >>= 1;
            }
            let mut o =
                values[0] * (self.decode[2 * i + 1] - self.decode[2 * i]) + self.decode[2 * i];
            if o < self.range[2 * i] {
                o = self.range[2 * i];
            }
            if o > self.range[2 * i + 1] {
                o = self.range[2 * i + 1];
            }
            out.push(o);
        }
        Ok(out)
    }
}

fn interpolate(x: &f32, x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> f32 {
    y_min + (x - x_min) * ((y_max - y_min) / (x_max - x_min))
}

fn normalize(values: &[f32], limits: &[f32]) -> Result<Vec<f32>> {
    if values.len() * 2 != limits.len() {
        return Err(PdfError::Function(
            "SampleFunction normalize values size errror".to_string(),
        ));
    }
    let mut res = Vec::new();
    for (i, v) in values.iter().enumerate() {
        let lowerbound: f32 = limits
            .get(i * 2)
            .ok_or(PdfError::Function(
                "SampleFunction normailize get lowerbound error".to_string(),
            ))?
            .to_owned();
        let upperbound: f32 = limits
            .get(i * 2 + 1)
            .ok_or(PdfError::Function(
                "SampleFunction normailize get upperbound error".to_string(),
            ))?
            .to_owned();
        let normal = ((v - lowerbound) / (upperbound - lowerbound)).max(0.0);
        let normal = normal.min(1.0);
        res.push(normal);
    }
    Ok(res)
}
