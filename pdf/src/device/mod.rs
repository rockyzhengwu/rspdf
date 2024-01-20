use crate::errors::PDFResult;
use crate::geom::path::Path;
use crate::geom::rectangle::Rectangle;

pub mod image_device;
pub mod text;
pub mod trace;
use crate::page::text::PageText;

pub trait Device {
    fn begain_page(&mut self, page_num: &u32, media: &Rectangle, crop: &Rectangle);
    fn end_page(&mut self, page_num: &u32);
    fn start_text(&mut self);
    fn show_text(&mut self, textobj: &PageText) -> PDFResult<()>;
    fn end_text(&mut self);
    fn paint_path(&mut self, pathinfo: &Path) -> PDFResult<()>;
}
