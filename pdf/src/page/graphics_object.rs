use crate::page::image::Image;
use crate::page::page_path::PagePath;
use crate::page::text::Text;

#[derive(Debug)]
pub enum GraphicsObject {
    Path(PagePath),
    Text(Text),
    Image(Image),
}
