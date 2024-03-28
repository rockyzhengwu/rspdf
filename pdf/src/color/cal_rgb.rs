use crate::color::common::decode_point;
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub struct CalRGB {
    gamma: [f32; 3],
    white_point: [f32; 3],
    black_point: [f32; 3],
    matrix: [f32; 9],
}
impl Default for CalRGB {
    fn default() -> Self {
        CalRGB {
            gamma: [1.0, 1.0, 1.0],
            white_point: [1.0, 1.0, 1.0],
            black_point: [0.0, 0.0, 0.0],
            matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl CalRGB {
    pub fn try_new(obj: &PDFObject) -> PDFResult<Self> {
        let mut color = CalRGB::default();
        if let Some(g) = decode_point(obj, "Gamma")? {
            color.gamma = g;
        }
        if let Some(wp) = decode_point(obj, "WhitePoint")? {
            color.white_point = wp;
        }
        if let Some(bp) = decode_point(obj, "BlackPoint")? {
            color.black_point = bp;
        }
        if let Some(mat) = obj.get_value("Matrix") {
            let vs = mat.as_array()?;
            if vs.len() != 9 {
                return Err(PDFError::ColorError(format!(
                    "CalRGB matrix need 9 items got:{:?}",
                    vs
                )));
            }
            for i in 0..9 {
                let v = vs.get(i).unwrap().as_f32()?;
                color.matrix[i] = v;
            }
        }
        Ok(color)
    }
}
