use crate::errors::{PDFError, PDFResult};

use super::RGBValue;

#[derive(Debug, Clone, Default)]
pub struct DeviceCMYK {}

impl DeviceCMYK {
    pub fn new() -> Self {
        Self {}
    }

    pub fn number_of_components(&self) -> u8 {
        4
    }

    pub fn to_rgb(&self, bytes: &[f32]) -> PDFResult<RGBValue> {
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
        let r = (255.0 * (1.0 - (c + k).min(1.0))) as u8;
        let g = (255.0 * (1.0 - (m + k).min(1.0))) as u8;
        let b = (255.0 * (1.0 - (y + k).min(1.0))) as u8;
        Ok(RGBValue(r, g, b))
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<RGBValue>> {
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
