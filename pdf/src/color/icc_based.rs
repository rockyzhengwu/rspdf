use crate::color::icc_profile::ICCProfile;
use crate::color::{parse_colorspace, ColorSpace};
use crate::error::{PdfError, Result};
use crate::object::PdfObject;
use crate::xref::Xref;

use super::device_cmyk::DeviceCmyk;
use super::device_gray::DeviceGray;
use super::device_rgb::DeviceRgb;
use super::value::{ColorRgb, ColorValue};

#[derive(Debug, Clone)]
pub struct IccBased {
    n: u8,
    alternate: Box<ColorSpace>,
    range: Vec<f32>,
    profile: Option<ICCProfile>,
}

impl Default for IccBased {
    fn default() -> Self {
        IccBased {
            n: 1,
            alternate: Box::new(ColorSpace::DeviceGray(DeviceGray::new())),
            range: Vec::new(),
            profile: None,
        }
    }
}

impl IccBased {
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let color_stream = match obj {
            PdfObject::Array(array) => {
                if array.len() < 2 {
                    return Err(PdfError::Color(
                        "IccBased Color array need 2 param at least".to_string(),
                    ));
                }
                let cd = xref
                    .read_object(array.get(1).unwrap())?
                    .as_stream()
                    .map_err(|_| {
                        PdfError::Color(format!(
                            "IccBased need Streamgot:{:?}",
                            xref.read_object(array.get(1).unwrap())
                        ))
                    })?
                    .to_owned();
                cd
            }
            PdfObject::Stream(s) => s.to_owned(),
            _ => {
                return Err(PdfError::Color("Bad IccBased Color".to_string()));
            }
        };

        let mut color = IccBased::default();
        if let Some(n) = color_stream.get_from_dict("N") {
            let nv = n
                .integer()
                .map_err(|_| PdfError::Color(format!("IccBased N is not a number:{:?}", n)))?;
            if !matches!(nv, 1 | 3 | 4) {
                return Err(PdfError::Color(format!(
                    "IccBased N is must 1,3,or 4 got: {:?}",
                    nv
                )));
            }
            color.n = nv as u8;
        } else {
            return Err(PdfError::Color(format!(
                "IccBased color need a N parameter "
            )));
        }
        if let Some(alt) = color_stream.get_from_dict("Alternate") {
            let altc = parse_colorspace(alt, xref)?;
            color.alternate = Box::new(altc);
        } else {
            match color.n {
                1 => {
                    color.alternate = Box::new(ColorSpace::DeviceGray(DeviceGray::new()));
                }
                3 => {
                    color.alternate = Box::new(ColorSpace::DeviceRgb(DeviceRgb::new()));
                }
                4 => {
                    color.alternate = Box::new(ColorSpace::DeviceCmyk(DeviceCmyk::new()));
                }
                _ => { // donothing}
                }
            }
        }
        match color_stream.get_from_dict("Range") {
            Some(r) => {
                let ra = r.as_array().map_err(|_| {
                    PdfError::Color(format!("IccBased Color need an array got :{:?}", r))
                })?;
                if (ra.len() as u8) != (color.n * 2_u8) {
                    return Err(PdfError::Color(format!(
                        "IccBased range array element is not valid got:{:?}",
                        ra.len()
                    )));
                }
                for i in 0..color.n * 2 {
                    let v = ra.get(i as usize).unwrap().as_number()?.real();
                    color.range.push(v);
                }
            }
            None => {
                for _ in 0..color.n {
                    color.range.push(0.0);
                    color.range.push(1.0);
                }
            }
        }

        Ok(color)
    }

    pub fn default_value(&self) -> ColorValue {
        match self.n {
            1 => ColorValue::new(vec![0.0]),
            3 => ColorValue::new(vec![0.0, 0.0, 0.0]),
            4 => ColorValue::new(vec![0.0, 0.0, 0.0, 0.0]),
            _ => {
                panic!("IccBased color n must be 1, 3, or 4")
            }
        }
    }
    pub fn rgb(&self, value: &ColorValue) -> Result<ColorRgb> {
        self.alternate.rgb(value)
    }

    pub fn number_of_components(&self) -> usize {
        self.n as usize
    }
    pub fn range(&self) -> &[f32] {
        self.range.as_slice()
    }
}
