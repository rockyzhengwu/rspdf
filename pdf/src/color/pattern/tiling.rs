use crate::error::{PdfError, Result};
use crate::geom::coordinate::Matrix;
use crate::geom::rect::Rect;
use crate::object::PdfObject;
use crate::xref::Xref;

#[derive(Debug, Clone, Default)]
pub struct TilingPattern {
    paint_type: u8,
    tiling_type: u8,
    bbox: Rect,
    xstep: i32,
    ystep: i32,
    matrix: Option<Matrix>,
}

impl TilingPattern {
    pub fn try_new(obj: &PdfObject, xref: &Xref) -> Result<Self> {
        let mut tp = TilingPattern::default();
        if let Some(pt) = obj.get_from_dict("PaintType") {
            let pt = pt
                .as_number()
                .map_err(|_| PdfError::Color("Pattern paint type is not a number".to_string()))?
                .integer();
            tp.paint_type = pt as u8;
        }

        if let Some(t) = obj.get_from_dict("TilingType") {
            let t = t
                .as_number()
                .map_err(|_| PdfError::Color("Pattern TilingType is not a number".to_string()))?
                .integer();
            tp.tiling_type = t as u8;
        }
        if let Some(bbox) = obj.get_from_dict("BBox") {
            let bbox = bbox
                .as_array()
                .map_err(|_| PdfError::Color("Pattern bbox is not an array".to_string()))?;
            let rect = Rect::new_from_pdf_bbox(bbox)?;
            tp.bbox = rect;
        }
        if let Some(x) = obj.get_from_dict("XStep") {
            let xs = x.as_number()?.integer();
            tp.xstep = xs;
        }
        if let Some(y) = obj.get_from_dict("YStep") {
            let ys = y.as_number()?.integer();
            tp.ystep = ys;
        }
        if let Some(matrix) = obj.get_from_dict("Matrix") {
            let m = matrix
                .as_array()
                .map_err(|_| PdfError::Color("Pattern Matrix is not an array".to_string()))?;
            let a = m
                .get(0)
                .ok_or(PdfError::Color("Matrix element error".to_string()))?
                .as_number()
                .map_err(|_| PdfError::Color("Matrix element is not a number".to_string()))?
                .real();
            let b = m
                .get(1)
                .ok_or(PdfError::Color("Matrix element error".to_string()))?
                .as_number()
                .map_err(|_| PdfError::Color("Matrix element is not a number".to_string()))?
                .real();
            let c = m
                .get(2)
                .ok_or(PdfError::Color("Matrix element error".to_string()))?
                .as_number()
                .map_err(|_| PdfError::Color("Matrix element is not a number".to_string()))?
                .real();
            let d = m
                .get(3)
                .ok_or(PdfError::Color("Matrix element error".to_string()))?
                .as_number()
                .map_err(|_| PdfError::Color("Matrix element is not a number".to_string()))?
                .real();
            let e = m
                .get(4)
                .ok_or(PdfError::Color("Matrix element error".to_string()))?
                .as_number()
                .map_err(|_| PdfError::Color("Matrix element is not a number".to_string()))?
                .real();
            let f = m
                .get(5)
                .ok_or(PdfError::Color("Matrix element error".to_string()))?
                .as_number()
                .map_err(|_| PdfError::Color("Matrix element is not a number".to_string()))?
                .real();
            tp.matrix = Some(Matrix::new(a, b, c, d, e, f));
        }

        if let Some(res) = obj.get_from_dict("Resources") {
            //println!("{:?}", res);
        }
        let content = obj.as_stream().unwrap().decode_data(Some(xref))?;
        Ok(tp)
    }
}
