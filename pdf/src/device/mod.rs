pub mod image_device;
pub mod text;
pub mod trace;
use crate::errors::PDFResult;
use crate::page::graphics_object::GraphicsObject;

pub trait Device {
    fn process(&mut self, obj: &GraphicsObject)->PDFResult<()>;
}
