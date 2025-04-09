use crate::{
    error::{PdfError, Result},
    geom::coordinate::Point,
    object::array::PdfArray,
};

#[derive(Debug, Default, Clone)]
pub struct Rect {
    lf: Point,
    width: f32,
    height: f32,
}

impl Rect {
    pub fn new(lf: Point, width: f32, height: f32) -> Self {
        Self { lf, width, height }
    }

    pub fn new_from_pdf_bbox(bbox: &PdfArray) -> Result<Self> {
        let llx = bbox
            .get(0)
            .ok_or(PdfError::Pattern("Rect from array llx is None".to_string()))?
            .as_number()
            .map_err(|_| PdfError::Pattern("Rect array element is not a number".to_string()))?
            .real();
        let lly = bbox
            .get(1)
            .ok_or(PdfError::Pattern("Rect from array lly is None".to_string()))?
            .as_number()
            .map_err(|_| PdfError::Pattern("Rect array element is not a number".to_string()))?
            .real();
        let urx = bbox
            .get(2)
            .ok_or(PdfError::Pattern("Rect from array urx is None".to_string()))?
            .as_number()
            .map_err(|_| PdfError::Pattern("Rect array elemnt is not a number".to_string()))?
            .real();
        let ury = bbox
            .get(3)
            .ok_or(PdfError::Pattern("Rect from array ury is NOne".to_string()))?
            .as_number()
            .map_err(|_| PdfError::Pattern("Rect array element is not number".to_string()))?
            .real();
        let rect = Rect::new(Point::new(llx, lly), urx - llx, ury - lly);
        Ok(rect)
    }

    pub fn lower_left(&self) -> &Point {
        &self.lf
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn lx(&self) -> f32 {
        self.lf.x()
    }
    pub fn ly(&self) -> f32 {
        self.lf.y()
    }
    pub fn ux(&self) -> f32 {
        self.lf.x() + self.width
    }
    pub fn uy(&self) -> f32 {
        self.lf.y() + self.height
    }
}
