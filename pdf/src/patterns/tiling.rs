use crate::{
    error::{PdfError, Result},
    geom::{coordinate::Matrix, rect::Rect},
    object::PdfObject,
};

#[derive(Debug, Clone, Default)]
pub struct TilingPattern {
    paint_type: u8,
    tiling_type: u8,
    bbox: Rect,
    xstep: f32,
    ystep: f32,
    matrix: Matrix,
}

impl TilingPattern {
    pub fn try_new(obj: &PdfObject) -> Result<Self> {
        let mut pattern = TilingPattern::default();
        let pt = obj
            .get_from_dict("PaintType")
            .ok_or(PdfError::Pattern(
                "TillingPattern paintType is None".to_string(),
            ))?
            .integer()
            .map_err(|_| PdfError::Pattern("PaintType is not a number".to_string()))?
            as u8;
        pattern.paint_type = pt;
        pattern.tiling_type = obj
            .get_from_dict("TilingType")
            .ok_or(PdfError::Pattern(
                "TillingPattern TilingType is None".to_string(),
            ))?
            .integer()
            .map_err(|_| PdfError::Pattern("TilingType is not a number".to_string()))?
            as u8;
        let bbox = obj
            .get_from_dict("BBox")
            .ok_or(PdfError::Pattern("TilingPattern Bbox is None".to_string()))?
            .as_array()
            .map_err(|_| PdfError::Pattern("TilingPattern BBox is not an array".to_string()))?;
        let bbox = Rect::new_from_pdf_bbox(bbox)
            .map_err(|e| PdfError::Pattern(format!("Create TilingPattern BBox error:{:?}", e)))?;
        pattern.bbox = bbox;
        if let Some(m) = obj.get_from_dict("Matrix") {
            let ma = m.as_array().map_err(|_| {
                PdfError::Pattern("TilingPattern Matrix is not an array".to_string())
            })?;
            let a = ma
                .get(0)
                .ok_or(PdfError::Pattern(
                    "TilingPattern Matrix element is not number".to_string(),
                ))?
                .as_number()
                .map_err(|_| {
                    PdfError::Pattern("TilingPattern Matrix element is not number".to_string())
                })?
                .real();

            let b = ma
                .get(1)
                .ok_or(PdfError::Pattern(
                    "TilingPattern Matrix element is not number".to_string(),
                ))?
                .as_number()
                .map_err(|_| {
                    PdfError::Pattern("TilingPattern Matrix element is not number".to_string())
                })?
                .real();

            let c = ma
                .get(2)
                .ok_or(PdfError::Pattern(
                    "TilingPattern Matrix element is not number".to_string(),
                ))?
                .as_number()
                .map_err(|_| {
                    PdfError::Pattern("TilingPattern Matrix element is not number".to_string())
                })?
                .real();

            let d = ma
                .get(3)
                .ok_or(PdfError::Pattern(
                    "TilingPattern Matrix element is not number".to_string(),
                ))?
                .as_number()
                .map_err(|_| {
                    PdfError::Pattern("TilingPattern Matrix element is not number".to_string())
                })?
                .real();

            let e = ma
                .get(4)
                .ok_or(PdfError::Pattern(
                    "TilingPattern Matrix element is not number".to_string(),
                ))?
                .as_number()
                .map_err(|_| {
                    PdfError::Pattern("TilingPattern Matrix element is not number".to_string())
                })?
                .real();

            let f = ma
                .get(5)
                .ok_or(PdfError::Pattern(
                    "TilingPattern Matrix element is not number".to_string(),
                ))?
                .as_number()
                .map_err(|_| {
                    PdfError::Pattern("TilingPattern Matrix element is not number".to_string())
                })?
                .real();
            pattern.matrix = Matrix::new(a, b, c, d, e, f);
        }
        let _resources = obj.get_from_dict("Resources");
        Ok(pattern)
    }
}
