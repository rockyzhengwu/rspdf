use crate::{
    error::{PdfError, Result},
    object::PdfObject,
    xref::Xref,
};

use super::value::{ColorRgb, ColorValue};

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
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let color_dict = match obj {
            PdfObject::Array(array) => {
                if array.len() < 2 {
                    return Err(PdfError::Color(
                        "CalGray Color array need 2 param at least".to_string(),
                    ));
                }
                let cd = xref
                    .read_object(array.get(1).unwrap())?
                    .as_dict()
                    .map_err(|_| PdfError::Color("CalGray need dict".to_string()))?
                    .to_owned();
                cd
            }
            PdfObject::Dict(d) => d.to_owned(),
            _ => {
                return Err(PdfError::Color("Bad CalGray Color".to_string()));
            }
        };

        let mut color = CalGray::default();
        if let Some(g) = color_dict.get("Gamma") {
            color.gamma = g.to_owned().as_number()?.real();
        }

        match color_dict.get("WhitePoint") {
            Some(wh) => {
                let wha = wh.as_array().map_err(|_| {
                    PdfError::Color(format!("CalGray WhitePoint is not an array got:{:?}", obj))
                })?;
                if wha.len() != 3 {
                    return Err(PdfError::Color(format!(
                        "CalGray WhitePoint need 3 elements :{:?}",
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
                                "CalGray WhitePoint {:?} value is not an number:{:?}",
                                i,
                                wha.get(i)
                            ))
                        })?
                        .real();
                    color.white_point[i] = v;
                }
            }
            None => {
                return Err(PdfError::Color(
                    "CalGray WhitePoint is required".to_string(),
                ));
            }
        }
        if let Some(bp) = color_dict.get("BlackPoint") {
            let bpa = bp.as_array().map_err(|_| {
                PdfError::Color(format!("CalGray BlackPoint is not an array got:{:?}", obj))
            })?;
            if bpa.len() != 3 {
                return Err(PdfError::Color(format!(
                    "CalGray BlackPoint need 3 elements :{:?}",
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
                            "CalGray BlackPoint {:?} value is not an number:{:?}",
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

    pub fn number_of_components(&self) -> usize {
        1
    }

    pub fn default_value(&self) -> ColorValue {
        ColorValue::new(vec![0.0])
    }

    pub fn rgb(&self, value: &ColorValue) -> Result<ColorRgb> {
        let a = value.values()[0];
        let x = self.white_point[0] * a.powf(self.gamma);
        let y = self.white_point[1] * a.powf(self.gamma);
        let z = self.white_point[2] * a.powf(self.gamma);
        let r = 3.2406 * x - 1.5372 * y - 0.4986 * z;
        let g = -0.9689 * x + 1.8758 * y + 0.0415 * z;
        let b = 0.0557 * x - 0.2040 * y + 1.0570 * z;
        Ok(ColorRgb::new(r, g, b))
    }
}
