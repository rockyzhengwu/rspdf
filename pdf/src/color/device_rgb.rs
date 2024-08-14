use std::u8;

use crate::errors::PDFResult;

use super::RGBValue;

#[derive(Debug, Clone, Default)]
pub struct DeviceRGB {}

impl DeviceRGB {
    pub fn new() -> Self {
        DeviceRGB {}
    }
    pub fn to_rgb(&self, inputs: &[f32]) -> PDFResult<RGBValue> {
        let r = inputs.first().unwrap().to_owned() as u8;
        let g = inputs.get(1).unwrap().to_owned() as u8;
        let b = inputs.last().unwrap().to_owned() as u8;
        Ok(RGBValue(r, g, b))
    }

    pub fn number_of_components(&self) -> u8 {
        3
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<RGBValue>> {
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
