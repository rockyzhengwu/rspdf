use crate::geom::path::Path;
use crate::page::image::Image;
use crate::page::text::Text;

#[derive(Debug)]
pub enum GraphicsObject {
    Path(Path),
    Text(Text),
    Image(Image),
}
