use crate::color::ColorRGBValue;
use crate::errors::{PDFError, PDFResult};

#[derive(Debug, Clone)]
pub struct DeviceCMYK {
    c: f32,
    m: f32,
    y: f32,
    k: f32,
}

impl Default for DeviceCMYK {
    fn default() -> Self {
        DeviceCMYK {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 0.0,
        }
    }
}

impl DeviceCMYK {
    pub fn new(c: f32, m: f32, y: f32, k: f32) -> Self {
        Self { c, m, y, k }
    }

    pub fn set_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) {
        self.c = c;
        self.m = m;
        self.y = y;
        self.k = k;
    }

    pub fn number_of_components(&self) -> u8 {
        4
    }

    pub fn to_rgb(&self, bytes: &[f32]) -> PDFResult<ColorRGBValue> {
        if bytes.len() != 4 {
            return Err(PDFError::ColorError(format!(
                "device cmykk params error:{:?}",
                bytes
            )));
        }

        let c = bytes.first().unwrap().to_owned() / 255.0;
        let m = bytes.get(1).unwrap().to_owned() / 255.0;
        let y = bytes.get(2).unwrap().to_owned() / 255.0;
        let k = bytes.get(3).unwrap().to_owned() / 255.0;
        let r = (255.0 * (1.0 - (c + k).min(1.0))) as u32;
        let g = (255.0 * (1.0 - (m + k).min(1.0))) as u32;
        let b = (255.0 * (1.0 - (y + k).min(1.0))) as u32;
        Ok(ColorRGBValue(r, g, b))
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<ColorRGBValue>> {
        let mut image = Vec::new();
        for chunk in bytes.chunks(4) {
            let inputs: Vec<f32> = chunk
                .to_owned()
                .iter()
                .map(|x| (x.to_owned() as f32))
                .collect();
            let rgb = self.to_rgb(inputs.as_slice())?;
            image.push(rgb)
        }
        Ok(image)
    }
}
