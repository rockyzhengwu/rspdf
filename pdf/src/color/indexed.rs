use std::io::{Read, Seek};

use crate::color::device_gray::DeviceGray;
use crate::color::{create_colorspace, ColorSpace};
use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFArray;

use super::{CMYKValue, GrayValue, RGBValue};

#[derive(Debug, Clone)]
pub struct Indexed {
    base: Box<ColorSpace>,
    hival: u8,
    lookup: Vec<u8>,
}

impl Default for Indexed {
    fn default() -> Self {
        Indexed {
            base: Box::new(ColorSpace::DeviceGray(DeviceGray {})),
            hival: 0,
            lookup: Vec::new(),
        }
    }
}

impl Indexed {
    pub fn try_new<T: Seek + Read>(obj: &PDFArray, doc: &Document<T>) -> PDFResult<Self> {
        // NOTE: base is a Name or Array not implement Array
        if obj.len() != 4 {
            return Err(PDFError::ColorError(format!(
                "Indexed colorspace param need 4 element got:{:?}",
                obj
            )));
        }
        // TODO: fix unwrap
        let base = obj.get(1).unwrap();
        let base = create_colorspace(base, doc)?;
        let base = Box::new(base);
        let hival = obj.get(2).unwrap().as_u8()?;
        let lookup_stream = doc.get_object_without_indriect(obj.last().unwrap())?;
        let lookup = lookup_stream.bytes()?;
        Ok(Indexed {
            base,
            hival,
            lookup,
        })
    }

    pub fn number_of_components(&self) -> u8 {
        1
    }

    pub fn to_rgb(&self, inputs: &[f32]) -> PDFResult<RGBValue> {
        match self.base.as_ref() {
            ColorSpace::DeviceRGB(rgb) => {
                let index = inputs[0].to_owned() as usize;
                let r = self.lookup[index] as f32;
                let g = self.lookup[index + 1] as f32;
                let b = self.lookup[index + 2] as f32;
                rgb.to_rgb(&[r, g, b])
            }
            ColorSpace::DeviceCMYK(cmyk) => {
                let index = inputs[0].to_owned() as usize;
                let c = self.lookup[index] as f32;
                let m = self.lookup[index + 1] as f32;
                let y = self.lookup[index + 2] as f32;
                let k = self.lookup[index + 3] as f32;
                cmyk.to_rgb(&[c, m, y, k])
            }
            _ => Err(PDFError::ColorError(
                "Indexed color not suport space".to_string(),
            )),
        }
    }
    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<RGBValue>> {
        let mut image = Vec::new();
        for byte in bytes {
            let p = byte.to_owned() as f32;
            let p = p * self.base.number_of_components() as f32;
            let inputs = vec![p];
            let rgb = self.to_rgb(inputs.as_slice())?;
            image.push(rgb);
        }
        Ok(image)
    }
    fn to_gray(&self, value: &super::ColorValue) -> PDFResult<GrayValue> {
        unimplemented!()
    }
    fn to_cmyk(&self, value: &super::ColorValue) -> PDFResult<CMYKValue> {
        unimplemented!()
    }
}
