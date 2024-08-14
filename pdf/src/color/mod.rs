use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

//pub mod cal_gray;
pub mod cal_gray;
pub mod cal_rgb;
pub mod device_cmyk;
pub mod device_gray;
pub mod device_rgb;
pub mod devicen;
pub mod iccbased;
pub mod indexed;
pub mod lab;
pub mod pattern;
pub mod separation;

mod common;

use cal_gray::CalGray;
use cal_rgb::CalRGB;
use device_cmyk::DeviceCMYK;
use device_gray::DeviceGray;
use device_rgb::DeviceRGB;
use devicen::DeviceN;
use iccbased::IccBased;
use indexed::Indexed;
use lab::Lab;
use pattern::Pattern;
use separation::Separation;

pub struct RGBValue(u8, u8, u8);
pub struct CMYKValue(u8, u8, u8, u8);
pub struct GrayValue(u8);

impl RGBValue {
    pub fn r(&self) -> u8 {
        self.0
    }
    pub fn g(&self) -> u8 {
        self.1
    }
    pub fn b(&self) -> u8 {
        self.2
    }
}

#[derive(Debug, Clone, Default)]
pub struct ColorValue {
    values: Vec<f32>,
}

impl ColorValue {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }
    pub fn values(&self) -> &[f32] {
        self.values.as_slice()
    }
}

#[derive(Debug, Clone)]
pub enum ColorSpace {
    DeviceGray(DeviceGray),
    DeviceRGB(DeviceRGB),
    DeviceCMYK(DeviceCMYK),
    CalGray(Box<CalGray>),
    CalRGB(Box<CalRGB>),
    Lab(Box<Lab>),
    ICCBased(Box<IccBased>),
    Separation(Box<Separation>),
    DeviceN(Box<DeviceN>),
    Indexed(Box<Indexed>),
    Pattern(Box<Pattern>),
}

pub fn create_colorspace<T: Seek + Read>(
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<ColorSpace> {
    match obj {
        PDFObject::Name(name) => match name.name() {
            "DeviceGray" | "G" => Ok(ColorSpace::new_device_gray()),
            "DeviceRGB" | "RGB" => Ok(ColorSpace::new_device_rgb()),
            "DeviceCMYK" => Ok(ColorSpace::new_device_cmyk()),
            _ => Err(PDFError::ColorError(format!(
                "colorspace {:?} not implement ",
                name
            ))),
        },
        PDFObject::Arrray(arr) => {
            let first = arr.first().unwrap().as_string().unwrap();
            match first.as_str() {
                "Lab" => {
                    let lab = Lab::try_new(arr, doc)?;
                    Ok(ColorSpace::Lab(Box::new(lab)))
                }
                "ICCBased" => {
                    let stream = arr.get(1).unwrap();
                    let stream = doc.get_object_without_indriect(stream).unwrap();
                    let iccbased = IccBased::try_new(&stream, doc)?;
                    Ok(ColorSpace::ICCBased(Box::new(iccbased)))
                }
                "Separation" => {
                    let separation = Separation::try_new(arr, doc)?;
                    Ok(ColorSpace::Separation(Box::new(separation)))
                }
                "Indexed" => {
                    let indexed = Indexed::try_new(arr, doc)?;
                    Ok(ColorSpace::Indexed(Box::new(indexed)))
                }
                "DeviceGray" => Ok(ColorSpace::new_device_gray()),
                "DeviceRGB" => Ok(ColorSpace::new_device_rgb()),
                "DeviceCMYK" => Ok(ColorSpace::new_device_cmyk()),

                _ => Err(PDFError::ColorError("colorspace not implement".to_string())),
            }
        }
        _ => Err(PDFError::ColorError(
            "create_colorspace need a Name or Array".to_string(),
        )),
    }
}

impl ColorSpace {
    pub fn number_of_components(&self) -> u8 {
        match self {
            ColorSpace::ICCBased(ref c) => c.number_of_components(),
            ColorSpace::Indexed(ref sc) => sc.number_of_components(),
            ColorSpace::DeviceRGB(ref sc) => sc.number_of_components(),
            ColorSpace::DeviceGray(ref sc) => sc.number_of_components(),
            ColorSpace::DeviceCMYK(ref cmyk) => cmyk.number_of_components(),
            _ => {
                panic!("not implement:{:?}", self)
            }
        }
    }

    pub fn new_device_rgb() -> Self {
        ColorSpace::DeviceRGB(DeviceRGB::new())
    }

    pub fn new_device_gray() -> Self {
        ColorSpace::DeviceGray(DeviceGray::new())
    }

    pub fn new_device_cmyk() -> Self {
        ColorSpace::DeviceCMYK(DeviceCMYK::new())
    }

    fn to_rgb(&self, value: &[f32]) -> PDFResult<RGBValue> {
        //*self.to_rgb(value)
        match self {
            ColorSpace::ICCBased(c) => c.as_ref().to_rgb(value),
            ColorSpace::Indexed(ref sc) => sc.as_ref().to_rgb(value),
            ColorSpace::DeviceRGB(ref sc) => sc.to_rgb(value),
            ColorSpace::DeviceGray(ref sc) => sc.to_rgb(value),
            ColorSpace::DeviceCMYK(ref cmyk) => cmyk.to_rgb(value),
            _ => {
                panic!("not implement:{:?}", self)
            }
        }
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<RGBValue>> {
        //pass
        match self {
            ColorSpace::Separation(ref s) => s.to_rgb_image(bytes),
            ColorSpace::ICCBased(ref c) => c.to_rgb_image(bytes),
            ColorSpace::Indexed(ref sc) => sc.to_rgb_image(bytes),
            ColorSpace::DeviceRGB(ref sc) => sc.to_rgb_image(bytes),
            ColorSpace::DeviceGray(ref sc) => sc.to_rgb_image(bytes),
            ColorSpace::DeviceCMYK(ref sc) => sc.to_rgb_image(bytes),
            _ => {
                panic!("not implement:{:?}", self)
            }
        }
    }
}
