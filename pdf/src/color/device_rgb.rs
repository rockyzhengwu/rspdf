use crate::color::ColorRGBValue;
use crate::errors::PDFResult;

#[derive(Debug, Clone)]
pub struct DeviceRGB {
    r: f32,
    g: f32,
    b: f32,
}

impl Default for DeviceRGB {
    fn default() -> Self {
        DeviceRGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

impl DeviceRGB {
    pub fn set_rgb(&mut self, r: f32, g: f32, b: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
    }

    pub fn to_rgb(&self, inputs: &[f32]) -> PDFResult<ColorRGBValue> {
        let r = inputs.first().unwrap().to_owned() as u32;
        let g = inputs.get(1).unwrap().to_owned() as u32;
        let b = inputs.last().unwrap().to_owned() as u32;
        Ok(ColorRGBValue(r, g, b))
    }
}
