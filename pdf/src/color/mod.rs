use std::fmt::Display;

use crate::{
    color::value::ColorRgb,
    error::{PdfError, Result},
    object::PdfObject,
    xref::Xref,
};

pub mod cal_gray;
pub mod cal_rgb;
pub mod device_cmyk;
pub mod device_gray;
pub mod device_rgb;
pub mod devicen;
pub mod icc_based;
pub mod icc_profile;
pub mod indexed;
pub mod lab;
pub mod pattern;
pub mod separation;
pub mod value;

use cal_gray::CalGray;
use cal_rgb::CalRgb;
use device_cmyk::DeviceCmyk;
use device_gray::DeviceGray;
use device_rgb::DeviceRgb;
use devicen::DeviceN;
use icc_based::IccBased;
use indexed::Indexed;
use lab::Lab;
use pattern::PatternColorSpace;
use separation::Separation;
use value::ColorValue;

#[derive(Debug, Clone)]
pub enum ColorSpace {
    DeviceGray(DeviceGray),
    DeviceRgb(DeviceRgb),
    DeviceCmyk(DeviceCmyk),
    CalGray(CalGray),
    CalRgb(CalRgb),
    Lab(Lab),
    IccBased(IccBased),
    Pattern(PatternColorSpace),
    Indexed(Indexed),
    Separation(Separation),
    DeviceN(DeviceN),
}

impl ColorSpace {
    pub fn rgb(&self, value: &ColorValue) -> Result<ColorRgb> {
        match self {
            ColorSpace::DeviceGray(gray) => gray.rgb(value),
            ColorSpace::DeviceRgb(rgb) => rgb.rgb(value),
            ColorSpace::DeviceCmyk(cmyk) => cmyk.rgb(value),
            ColorSpace::Lab(lab) => lab.rgb(value),
            ColorSpace::IccBased(icc) => icc.rgb(value),
            ColorSpace::CalGray(cg) => cg.rgb(value),
            ColorSpace::CalRgb(cr) => cr.rgb(value),
            ColorSpace::Indexed(indexed) => indexed.rgb(value),
            ColorSpace::Separation(sep) => sep.rgb(value),
            _ => {
                unimplemented!("not implent rgb of colorspace:{:?}", self)
            }
        }
    }
    pub fn default_value(&self) -> ColorValue {
        match self {
            ColorSpace::DeviceGray(gray) => gray.default_value(),
            ColorSpace::DeviceRgb(rgb) => rgb.default_value(),
            ColorSpace::DeviceCmyk(cmyk) => cmyk.default_value(),
            ColorSpace::Lab(lab) => lab.default_value(),
            ColorSpace::IccBased(icc) => icc.default_value(),
            ColorSpace::CalGray(cg) => cg.default_value(),
            ColorSpace::CalRgb(cr) => cr.default_value(),
            ColorSpace::Indexed(indexed) => indexed.default_value(),
            ColorSpace::Separation(sep) => sep.default_value(),
            ColorSpace::Pattern(p) => p.default_value(),
            _ => {
                unimplemented!("not implement default_value of colorspace:{:?}", self)
            }
        }
    }

    pub fn number_of_components(&self) -> usize {
        match self {
            ColorSpace::DeviceGray(gray) => gray.number_of_components(),
            ColorSpace::DeviceRgb(rgb) => rgb.number_of_components(),
            ColorSpace::DeviceCmyk(cmyk) => cmyk.number_of_components(),
            ColorSpace::Lab(lab) => lab.number_of_components(),
            ColorSpace::IccBased(icc) => icc.number_of_components(),
            ColorSpace::CalGray(cg) => cg.number_of_components(),
            ColorSpace::CalRgb(cr) => cr.number_of_components(),
            ColorSpace::Indexed(indexed) => indexed.number_of_components(),
            ColorSpace::Separation(sep) => sep.number_of_components(),
            _ => {
                unimplemented!("not implement number_of_components  : {:?}", self)
            }
        }
    }
}

pub fn parse_colorspace(obj: &PdfObject, xref: &Xref) -> Result<ColorSpace> {
    match obj {
        PdfObject::Name(name) => match name.name() {
            "G" | "DeviceGray" => Ok(ColorSpace::DeviceGray(DeviceGray::new())),
            "RGB" | "DeviceRGB" => Ok(ColorSpace::DeviceRgb(DeviceRgb::new())),
            "CMYK" | "DeviceCMYK" => Ok(ColorSpace::DeviceCmyk(DeviceCmyk::new())),
            "Pattern" => Ok(ColorSpace::Pattern(PatternColorSpace::default())),
            _ => return Err(PdfError::Color(format!("Color name is error:{:?}", name))),
        },
        PdfObject::Array(array) => {
            let cn = array
                .get(0)
                .ok_or(PdfError::Color("ColorSpace array is empty".to_string()))?
                .as_name()
                .map_err(|_| PdfError::Color("ColorSpace new need an array".to_string()))?;
            match cn.name() {
                "DeviceGray" => Ok(ColorSpace::DeviceGray(DeviceGray::new())),
                "DeviceRGB" => Ok(ColorSpace::DeviceRgb(DeviceRgb::new())),
                "DeviceCMYK" => Ok(ColorSpace::DeviceCmyk(DeviceCmyk::new())),
                "CalGray" => Ok(ColorSpace::CalGray(CalGray::try_new(obj, xref)?)),
                "CalRGB" => Ok(ColorSpace::CalRgb(CalRgb::try_new(obj, xref)?)),
                "Pattern" => Ok(ColorSpace::Pattern(PatternColorSpace::default())),
                "Indexed" => Ok(ColorSpace::Indexed(Indexed::try_new(array, xref)?)),
                "Separation" => Ok(ColorSpace::Separation(Separation::try_new(array, xref)?)),
                "ICCBased" => Ok(ColorSpace::IccBased(IccBased::try_new(obj, xref)?)),
                "Lab" => Ok(ColorSpace::Lab(Lab::try_new(obj)?)),
                _ => {
                    unimplemented!()
                }
            }
        }
        PdfObject::Indirect(_) => {
            let nobj = xref.read_object(obj)?;
            parse_colorspace(&nobj, xref)
        }
        _ => {
            return Err(PdfError::Color(format!(
                "Parse ColorSpace need an PdfArray or PdfName got :{:?}",
                obj
            )));
        }
    }
}

impl Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorSpace::DeviceGray(_) => write!(f, "DeviceGray"),
            ColorSpace::DeviceRgb(_) => write!(f, "DeviceRGB"),
            ColorSpace::DeviceCmyk(_) => write!(f, "DeviceCMYK"),
            ColorSpace::Lab(_) => write!(f, "Lab"),
            ColorSpace::IccBased(_) => write!(f, "ICCBased"),
            ColorSpace::CalGray(_) => write!(f, "CalGray"),
            ColorSpace::CalRgb(_) => write!(f, "CalRGB"),
            ColorSpace::Indexed(_) => write!(f, "Indexed"),
            ColorSpace::Separation(_) => write!(f, "Separation"),
            ColorSpace::Pattern(_) => write!(f, "Pattern"),
            ColorSpace::DeviceN(_) => write!(f, "DeviceN"),
        }
    }
}
