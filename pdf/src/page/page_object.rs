use crate::geom::path::Path;
use crate::page::image::Image;
use crate::page::text::PageText;

pub enum PageObject<'a> {
    Path(Path),
    Text(PageText<'a>),
    Image(Image),
}
