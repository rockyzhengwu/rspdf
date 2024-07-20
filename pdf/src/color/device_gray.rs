use crate::color::ColorRGBValue;
use crate::errors::PDFResult;

#[derive(Debug, Clone)]
pub struct DeviceGray {
    gray: f32,
}
impl Default for DeviceGray {
    fn default() -> Self {
        DeviceGray { gray: 0.0 }
    }
}
impl DeviceGray {
    pub fn new(gray: f32) -> Self {
        Self { gray }
    }
    pub fn set_gray(&mut self, gray: f32) {
        self.gray = gray;
    }
    pub fn number_of_components(&self) -> u8 {
        1
    }
    pub fn to_rgb(&self, bytes: &[f32]) -> PDFResult<ColorRGBValue> {
        let v = bytes[0].to_owned() as u32;
        Ok(ColorRGBValue(v, v, v))
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<ColorRGBValue>> {
        let mut image = Vec::new();
        for b in bytes {
            let v = b.to_owned() as u32;
            let rgb = ColorRGBValue(v, v, v);
            image.push(rgb);
        }
        Ok(image)
    }
}
