pub mod image_device;
pub mod text;
pub mod trace;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;
use crate::page::graphics_object::GraphicsObject;

pub trait Device {
    fn start_page(&mut self, bbox:Rectangle);
    fn process(&mut self, obj: &GraphicsObject)->PDFResult<()>;
}
