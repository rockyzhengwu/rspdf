use crate::font::Font;
use crate::geom::matrix::Matrix;
use crate::geom::rectangle::Rectangle;

#[derive(Debug)]
pub struct TextItem {
    tm: Matrix,
    unicode: char,
    code: u32,
}

impl TextItem {
    pub fn new(tm: Matrix, unicode: char, code: u32) -> Self {
        TextItem { tm, unicode, code }
    }

    pub fn bbox(&self) -> Rectangle {
        let lx = self.tm.v31;
        let ly = self.tm.v32;
        // TODO calc ux, uy
        Rectangle::new(lx, ly, 0.0, 0.0)
    }

    pub fn unicode(&self) -> &char {
        &self.unicode
    }

    pub fn code(&self) -> &u32 {
        &self.code
    }
    pub fn tm(&self) -> &Matrix {
        &self.tm
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PageText<'a> {
    bbox: Rectangle,
    items: Vec<TextItem>,
    font: &'a Font,
    font_size: f64,
    ctm: Matrix,
}

impl<'a> PageText<'a> {
    pub fn new(items: Vec<TextItem>, font: &'a Font, font_size: f64, ctm: Matrix) -> Self {
        PageText {
            bbox: Rectangle::default(),
            items,
            font,
            font_size,
            ctm,
        }
    }

    pub fn items(&self) -> &[TextItem] {
        &self.items
    }

    pub fn font(&self) -> &Font {
        self.font
    }

    pub fn font_size(&self) -> &f64 {
        &self.font_size
    }
    pub fn ctm(&self) -> &Matrix {
        &self.ctm
    }
}
