use crate::color::common::decode_point;
use crate::errors::PDFResult;
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub struct CalGray {
    gamma: f32,
    white_point: [f32; 3],
    black_point: [f32; 3],
}

impl Default for CalGray {
    fn default() -> Self {
        CalGray {
            gamma: 1.0,
            white_point: [1.0, 1.0, 1.0],
            black_point: [0.0, 0.0, 0.0],
        }
    }
}

impl CalGray {
    pub fn try_new(obj: &PDFObject) -> PDFResult<Self> {
        let mut color = CalGray::default();
        if let Some(g) = obj.get_value("Gamma") {
            color.gamma = g.to_owned().as_f32()?;
        }
        if let Some(wp) = decode_point(obj, "WhitePoint")? {
            color.white_point = wp;
        }
        if let Some(bp) = decode_point(obj, "BlackPoint")? {
            color.black_point = bp;
        }
        Ok(color)
    }

    pub fn number_of_components(&self) -> u8 {
        1
    }
}
