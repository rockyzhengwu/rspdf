use crate::font::Font;
use crate::geom::rectangle::Rectangle;

#[derive(Debug)]
pub struct TextItem<'a> {
    bbox: Rectangle,
    font: &'a Font,
}

impl<'a> TextItem<'a> {
    pub fn new(lx: f64, ly: f64, font: &'a Font) -> Self {
        let bbox = Rectangle::new(lx, ly, 0.0, 0.0);
        TextItem { bbox, font }
    }
}

#[derive(Debug)]
pub struct PageText<'a> {
    bbox: Rectangle,
    items: Vec<TextItem<'a>>,
}

impl<'a> PageText<'a> {
    pub fn new(items: Vec<TextItem<'a>>) -> Self {
        PageText {
            bbox: Rectangle::default(),
            items,
        }
    }
}
