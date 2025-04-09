use super::value::{ColorRgb, ColorValue};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct DeviceCmyk {}

impl DeviceCmyk {
    pub fn new() -> Self {
        DeviceCmyk {}
    }
    pub fn default_value(&self) -> ColorValue {
        ColorValue::new(vec![0.0, 0.0, 0.0, 0.0])
    }

    pub fn number_of_components(&self) -> usize {
        4
    }
    pub fn rgb(&self, value: &ColorValue) -> Result<ColorRgb> {
        let c = value.values()[0];
        let m = value.values()[1];
        let y = value.values()[2];
        let k = value.values()[3];
        let r = 1.0 - (c + k).min(1.0);
        let g = 1.0 - (m + k).min(1.0);
        let b = 1.0 - (y + k).min(1.0);
        Ok(ColorRgb::new(r, g, b))
    }
}
