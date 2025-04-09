use crate::error::{PdfError, Result};
use crate::object::PdfObject;
use crate::xref::Xref;

use super::value::{ColorRgb, ColorValue};

#[derive(Debug, Clone)]
pub struct CalRgb {
    gamma: [f32; 3],
    white_point: [f32; 3],
    black_point: [f32; 3],
    matrix: [f32; 9],
}

impl Default for CalRgb {
    fn default() -> Self {
        CalRgb {
            gamma: [1.0, 1.0, 1.0],
            white_point: [1.0, 1.0, 1.0],
            black_point: [0.0, 0.0, 0.0],
            matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl CalRgb {
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let color_dict = match obj {
            PdfObject::Array(array) => {
                if array.len() < 2 {
                    return Err(PdfError::Color(
                        "CalRGB Color array need 2 param at least".to_string(),
                    ));
                }
                let cd = xref
                    .read_object(array.get(1).unwrap())?
                    .as_dict()
                    .map_err(|_| PdfError::Color("CalRGB need dict".to_string()))?
                    .to_owned();
                cd
            }
            PdfObject::Dict(d) => d.to_owned(),
            _ => {
                return Err(PdfError::Color("Bad CalRGB Color".to_string()));
            }
        };

        let mut color = CalRgb::default();
        if let Some(m) = color_dict.get("Matrix") {
            let ma = m
                .as_array()
                .map_err(|_| PdfError::Color(format!("CalRgb Matrix need a array got:{:?}", m)))?;
            if ma.len() != 9 {
                return Err(PdfError::Color(format!(
                    "CalRgb Matrix need 9 elements got:{:?}",
                    ma
                )));
            }
            for i in 0..9 {
                let v = ma.get(i).unwrap().as_number().unwrap().real();
                color.matrix[i] = v;
            }
        }
        if let Some(g) = color_dict.get("Gamma") {
            let ga = g.as_array().map_err(|_| {
                PdfError::Color(format!("CalRgb color Gamma is not a array:{:?}", g))
            })?;
            let r = ga
                .get(0)
                .ok_or(PdfError::Color(
                    "CalRgb WhitePoint first value is not exist".to_string(),
                ))?
                .as_number()?
                .real();

            let g = ga
                .get(1)
                .ok_or(PdfError::Color(
                    "CalRgb Gamma second value is not exist".to_string(),
                ))?
                .as_number()?
                .real();

            let b = ga
                .get(2)
                .ok_or(PdfError::Color(
                    "CalRgb Gamma third value is not exist".to_string(),
                ))?
                .as_number()?
                .real();
            color.gamma = [r, g, b];
        }
        match color_dict.get("WhitePoint") {
            Some(wh) => {
                let wha = wh.as_array().map_err(|_| {
                    PdfError::Color(format!("CalRgb WhitePoint is not an array got:{:?}", obj))
                })?;
                if wha.len() != 3 {
                    return Err(PdfError::Color(format!(
                        "CalRgb WhitePoint need 3 elements :{:?}",
                        wha
                    )));
                }
                for i in 0..3 {
                    let v = wha
                        .get(i)
                        .unwrap()
                        .as_number()
                        .map_err(|_| {
                            PdfError::Color(format!(
                                "CalRgb WhitePoint {:?} value is not an number:{:?}",
                                i,
                                wha.get(i)
                            ))
                        })?
                        .real();
                    color.white_point[i] = v;
                }
            }
            None => {
                return Err(PdfError::Color("CalRgb WhitePoint is required".to_string()));
            }
        }
        if let Some(bp) = color_dict.get("BlackPoint") {
            let bpa = bp.as_array().map_err(|_| {
                PdfError::Color(format!("CalRgb BlackPoint is not an array got:{:?}", obj))
            })?;
            if bpa.len() != 3 {
                return Err(PdfError::Color(format!(
                    "CalRgb BlackPoint need 3 elements :{:?}",
                    bpa
                )));
            }
            for i in 0..3 {
                let v = bpa
                    .get(i)
                    .unwrap()
                    .as_number()
                    .map_err(|_| {
                        PdfError::Color(format!(
                            "CalRgb BlackPoint {:?} value is not an number:{:?}",
                            i,
                            bpa.get(i)
                        ))
                    })?
                    .real();
                color.black_point[i] = v;
            }
        }
        Ok(color)
    }
    pub fn default_value(&self) -> ColorValue {
        ColorValue::new(vec![0.0, 0.0, 0.0])
    }

    pub fn number_of_components(&self) -> usize {
        3
    }

    pub fn rgb(&self, value: &ColorValue) -> Result<ColorRgb> {
        let a = value.values()[0];
        let b = value.values()[1];
        let c = value.values()[2];
        let gr = a.powf(self.gamma[0]);
        let gg = b.powf(self.gamma[1]);
        let gb = c.powf(self.gamma[2]);

        let x = self.matrix[0] * gr + self.matrix[3] * gg + self.matrix[6] * gb;
        let y = self.matrix[1] * gr + self.matrix[4] * gg + self.matrix[7] * gb;
        let z = self.matrix[2] * gr + self.matrix[5] * gg + self.matrix[8] * gb;
        let r = 3.2406 * x - 1.5372 * y - 0.4986 * z;
        let g = -0.9689 * x + 1.8758 * y + 0.0415 * z;
        let b = 0.0557 * x - 0.2040 * y + 1.0570 * z;
        Ok(ColorRgb::new(r, g, b))
    }
}
