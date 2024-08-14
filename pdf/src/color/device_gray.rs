use crate::errors::PDFResult;

use super::RGBValue;

#[derive(Debug, Clone, Default)]
pub struct DeviceGray {}

impl DeviceGray {
    pub fn new() -> Self {
        Self {}
    }
    pub fn number_of_components(&self) -> u8 {
        1
    }
    pub fn to_rgb(&self, bytes: &[f32]) -> PDFResult<RGBValue> {
        let v = bytes[0].to_owned() as u8;
        Ok(RGBValue(v, v, v))
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<RGBValue>> {
        let mut image = Vec::new();
        for b in bytes {
            let v = b.to_owned();
            let rgb = RGBValue(v, v, v);
            image.push(rgb);
        }
        Ok(image)
    }
}
