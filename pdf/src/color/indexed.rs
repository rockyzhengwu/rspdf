use std::io::{Read, Seek};

use crate::color::device_gray::DeviceGray;
use crate::color::ColorSpace;
use crate::document::Document;
use crate::errors::PDFResult;
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
        unimplemented!()
    }
}
