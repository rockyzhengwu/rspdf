use crate::canvas::path_info::PathInfo;
use crate::canvas::text_info::TextInfo;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;

pub mod image_device;
pub mod text;

pub trait Device {
    fn begain_page(&mut self, media: &Rectangle, crop: &Rectangle);
    fn end_page(&mut self);
    fn show_text(&mut self, textinfo: TextInfo) -> PDFResult<()>;
    fn paint_path(&mut self, pathinfo: PathInfo) -> PDFResult<()>;
    fn close(&mut self) -> PDFResult<()>;
}
