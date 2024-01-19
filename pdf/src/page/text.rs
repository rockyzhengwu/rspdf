use crate::font::Font;
use crate::geom::rectangle::Rectangle;

#[derive(Debug)]
pub struct TextItem {
    bbox: Rectangle,
    ch: char,
}

impl TextItem {
    pub fn new(lx: f64, ly: f64, ch: char) -> Self {
        let bbox = Rectangle::new(lx, ly, 0.0, 0.0);
        TextItem { bbox, ch }
    }
}

#[derive(Debug)]
pub struct PageText<'a> {
    bbox: Rectangle,
    items: Vec<TextItem>,
    font: &'a Font,
}

impl<'a> PageText<'a> {
    pub fn new(items: Vec<TextItem>, font: &'a Font) -> Self {
        PageText {
            bbox: Rectangle::default(),
            items,
            font,
        }
    }
}
