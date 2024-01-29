use crate::errors::PDFResult;
use crate::geom::path::Path;
use crate::geom::rectangle::Rectangle;

pub mod image_device;
pub mod text;
pub mod trace;
use crate::page::text::Text;

pub trait Device {
    fn begain_page(&mut self, page_num: &u32, media: Option<Rectangle>, crop: Option<Rectangle>);
    fn end_page(&mut self, page_num: &u32);
    fn start_text(&mut self);
    fn show_text(&mut self, textobj: &Text) -> PDFResult<()>;
    fn end_text(&mut self);
    fn paint_path(&mut self, pathinfo: &Path) -> PDFResult<()>;
}
