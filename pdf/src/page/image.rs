use std::io::Cursor;

use bitstream_io::{BigEndian, BitRead, BitReader};

use crate::{
    color::{
        device_gray::DeviceGray,
        parse_colorspace,
        value::{ColorRgb, ColorValue},
        ColorSpace,
    },
    error::{PdfError, Result},
    filter::apply_filter,
    object::{dictionary::PdfDict, stream::PdfStream},
    xref::Xref,
};

use super::graphics_state::RenderIntent;

#[derive(Debug, Clone, Default)]
pub struct PdfImage {
    width: f32,
    height: f32,
    color_space: Option<ColorSpace>,
    bpc: u8,
    intent: Option<RenderIntent>,
    image_mask: bool,
    decode: Vec<f32>,
    interpolate: bool,
    alternates: Vec<PdfImage>,
    is_inline: bool,
    data: Vec<u8>,
    name: String,
}

impl PdfImage {
    pub fn try_new(obj: &PdfStream, xref: &Xref) -> Result<Self> {
        let mut image = PdfImage::default();
        image.is_inline = false;
        if let Some(name) = obj.get_from_dict("Name") {
            image.name = name.as_name().unwrap().name().to_string();
        }

        image.data = obj
            .decode_data(Some(xref))
            .map_err(|e| PdfError::Image(format!("Decode PdfImage data error:{:?}", e)))?;

        if let Some(imask) = obj.get_from_dict("ImageMask") {
            image.image_mask = imask.as_bool()?.0;
        } else {
            image.image_mask = false;
        }

        let width = obj
            .get_from_dict("Width")
            .ok_or(PdfError::Image("PdfImage Width entry is None".to_string()))?
            .as_number()?
            .real();

        let height = obj
            .get_from_dict("Height")
            .ok_or(PdfError::Image("PdfImage Height entry is None".to_string()))?
            .as_number()?
            .real();
        image.width = width;
        image.height = height;
        if let Some(c) = obj.get_from_dict("ColorSpace") {
            let cs = parse_colorspace(c, xref)?;
            image.color_space = Some(cs);
        } else {
            if !image.image_mask {
                image.color_space = Some(ColorSpace::DeviceGray(DeviceGray::new()));
            }
        }

        if let Some(bits) = obj.get_from_dict("BitsPerComponent") {
            let bits = bits.integer()? as u8;
            image.bpc = bits;
            // TODO JPXDecode
            if image.image_mask && image.bpc != 1 {
                return Err(PdfError::Image(
                    "Mask iamge BitsPerComponent must be 1".to_string(),
                ));
            }
        }
        if let Some(intent) = obj.get_from_dict("Intent") {
            let intent_name = intent
                .as_name()
                .map_err(|_| PdfError::Image("PdfImage Intent is not a name".to_string()))?;
            image.intent = Some(RenderIntent::new_from_str(intent_name.name())?);
        }
        if let Some(ip) = obj.get_from_dict("Interpolate") {
            image.interpolate = ip
                .as_bool()
                .map_err(|_| PdfError::Image("PdfImage Interpolate is not a bool".to_string()))?
                .0;
        } else {
            image.interpolate = false;
        }
        if let Some(decode) = obj.get_from_dict("Decode") {
            let decode = decode
                .as_array()
                .map_err(|_| PdfError::Image("Image Decode array is not an array ".to_string()))?;
            for i in 0..decode.len() {
                let v = decode
                    .get(i)
                    .ok_or(PdfError::Image("Decode elmenet error".to_string()))?
                    .as_number()
                    .map_err(|_| {
                        PdfError::Image("Image Decode element not a number".to_string())
                    })?;
                image.decode.push(v.real());
            }
        } else {
            // TODO JPXDecode image decode can be None and without default value
            image.decode = image.default_decode();
        }

        Ok(image)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn try_new_inline(info: &PdfDict, data: &[u8], xref: &Xref) -> Result<Self> {
        let mut image = PdfImage::default();
        image.is_inline = true;
        if let Some(w) = info.get("W") {
            image.width = w
                .as_number()
                .map_err(|_| PdfError::Image("Inline image W is not a number".to_string()))?
                .real();
        }

        if let Some(h) = info.get("H") {
            image.height = h
                .as_number()
                .map_err(|_| PdfError::Image("Inline image H is not a number".to_string()))?
                .real();
        }
        if let Some(bpc) = info.get("BPC") {
            image.bpc = bpc
                .as_number()
                .map_err(|_| PdfError::Image("Inline image PBC is not a number".to_string()))?
                .integer() as u8;
        }
        if let Some(cs) = info.get("CS") {
            image.color_space = Some(parse_colorspace(cs, xref)?);
        }

        if let Some(d) = info.get("D") {
            let da = d
                .as_array()
                .map_err(|_| PdfError::Image("Inline image D is not an array".to_string()))?;
            for v in da.iter() {
                let vv = v
                    .as_number()
                    .map_err(|_| PdfError::Image("Decode element is not a number".to_string()))?
                    .real();
                image.decode.push(vv);
            }
        } else {
            image.decode = image.default_decode();
        }
        if let Some(f) = info.get("F") {
            let dp = match info.get("DP") {
                Some(p) => Some(p.as_dict()?),
                None => None,
            };
            let name = f.as_name()?.name();
            image.data = apply_filter(name, data, dp).unwrap();
        } else {
            image.data = data.to_vec();
        }
        Ok(image)
    }

    fn default_decode(&self) -> Vec<f32> {
        match self.color_space.as_ref() {
            Some(ColorSpace::DeviceGray(_)) => vec![0.0, 1.0],
            Some(ColorSpace::DeviceRgb(_)) => vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            Some(ColorSpace::DeviceCmyk(_)) => vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            Some(ColorSpace::CalGray(_)) => vec![0.0, 1.0],
            Some(ColorSpace::CalRgb(_)) => vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            Some(ColorSpace::Separation(_)) => vec![0.0, 1.0],
            Some(ColorSpace::Indexed(_)) => {
                let base: f32 = 2.0;
                vec![0.0, base.powf(self.bpc as f32) - 1.0]
            }
            Some(ColorSpace::Lab(lab)) => {
                let range = lab.range();
                let mut res = vec![0.0, 100.0];
                res.extend_from_slice(range);
                res
            }
            Some(ColorSpace::IccBased(icc)) => icc.range().to_vec(),
            Some(ColorSpace::DeviceN(_)) => {
                unimplemented!("DeviceN for image not implement");
            }
            Some(ColorSpace::Pattern(_)) => {
                panic!("Pattern colorspace is not permitted in pdf")
            }
            _ => {
                unimplemented!("default decode not implement for ColorSpace")
            }
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn color_space(&self) -> Option<&ColorSpace> {
        self.color_space.as_ref()
    }
    pub fn is_mask(&self) -> bool {
        self.image_mask
    }
    pub fn image_data(&self) -> &[u8] {
        self.data.as_slice()
    }
    fn decode_value(&self, values: &[f32]) -> Result<ColorValue> {
        if values.len() * 2 != self.decode.len() {
            return Err(PdfError::Image(
                "color value array length not half of decode array".to_string(),
            ));
        }
        let mut vs = Vec::new();
        let dmax = (2.0_f32).powf(self.bpc as f32) - 1.0;
        for (i, v) in values.iter().enumerate() {
            let vmin = self.decode[2 * i];
            let vmax = self.decode[2 * i + 1];
            let dv = vmin + (v * (vmax - vmin) / dmax);
            vs.push(dv);
        }
        Ok(ColorValue::new(vs))
    }

    pub fn rgb_image(&self) -> Result<Vec<ColorRgb>> {
        let mut bitreader = BitReader::endian(Cursor::new(self.data.to_vec()), BigEndian);
        if self.image_mask {
            let mut rgb_data = Vec::new();
            let bytes_per_row = ((self.width + 7.0) / 8.0).floor() as usize;
            let pad_bits = bytes_per_row * 8 - self.width as usize;
            for _h in 0..self.height as usize {
                for _w in 0..self.width as usize {
                    let v = bitreader
                        .read_bit()
                        .map_err(|e| PdfError::Image(format!("read image data error:{:?}", e)))?;
                    if v {
                        rgb_data.push(ColorRgb::new(255.0, 255.0, 255.0));
                    } else {
                        rgb_data.push(ColorRgb::new(0.0, 0.0, 0.0));
                    }
                }
                bitreader
                    .skip(pad_bits as u32)
                    .map_err(|_| PdfError::Image("Read Maks image error".to_string()))?;
            }
            return Ok(rgb_data);
        }

        let mut bitreader = BitReader::endian(Cursor::new(self.data.to_vec()), BigEndian);
        let bpc = self.bpc as u32;
        match &self.color_space {
            Some(cs) => {
                let mut rgb_data = Vec::new();
                let n = cs.number_of_components();
                for _h in 0..self.height as usize {
                    for _w in 0..self.width as usize {
                        let mut sa = Vec::new();
                        for _ in 0..n {
                            let v = bitreader.read::<u16>(bpc).map_err(|e| {
                                PdfError::Image(format!(
                                    "read data from image stream error:{:?}",
                                    e
                                ))
                            })?;
                            sa.push(v as f32);
                        }
                        let rgb_value = self.decode_value(sa.as_slice())?;
                        let rgb = cs.rgb(&rgb_value)?;
                        rgb_data.push(rgb);
                    }
                }
                Ok(rgb_data)
            }
            None => Err(PdfError::Image("PdfImage has no color_space".to_string())),
        }
    }
    pub fn is_inline(&self) -> bool {
        self.is_inline
    }
}
