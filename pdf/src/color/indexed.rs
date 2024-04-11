use std::io::{Read, Seek};

use crate::color::device_gray::DeviceGray;
use crate::color::{create_colorspace, ColorRGBValue, ColorSpace};
use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFArray;

#[derive(Debug, Clone)]
pub struct Indexed {
    base: Box<ColorSpace>,
    hival: u8,
    lookup: Vec<u8>,
}

impl Default for Indexed {
    fn default() -> Self {
        Indexed {
            base: Box::new(ColorSpace::DeviceGray(DeviceGray::default())),
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

    pub fn to_rgb(&self, inputs: &[f32]) -> PDFResult<ColorRGBValue> {
        let index = inputs[0].to_owned() as usize;
        let r = self.lookup[index] as u32;
        let g = self.lookup[index + 1] as u32;
        let b = self.lookup[index + 2] as u32;
        Ok(ColorRGBValue(r, g, b))
    }
    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<ColorRGBValue>> {
        let mut image = Vec::new();
        for byte in bytes {
            let p = byte.to_owned() as f32;
            let inputs = vec![p];
            let rgb = self.to_rgb(inputs.as_slice())?;
            image.push(rgb);
        }
        Ok(image)
    }
}
