use crate::geom::path::Path;
use crate::page::image::Image;
use crate::page::text::Text;

pub enum PageObject<'a> {
    Path(Path),
    Text(Text<'a>),
    Image(Image),
}
