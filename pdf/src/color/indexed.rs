use crate::color::device_gray::DeviceGray;
use crate::color::ColorSpace;

#[derive(Debug)]
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
    
}
