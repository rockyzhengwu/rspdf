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
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

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

    pub fn number_of_components(&self) -> u8 {
        3
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<ColorRGBValue>> {
        let mut image = Vec::new();

        for chunk in bytes.chunks(3) {
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
