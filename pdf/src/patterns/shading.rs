use crate::{error::Result, geom::coordinate::Matrix, object::PdfObject};

#[derive(Debug, Clone)]
pub enum Shading {}

#[derive(Debug, Clone)]
pub struct ShadingPattern {
    matrix: Matrix,
    shading: Shading,
}

impl ShadingPattern {
    pub fn try_new(obj: &PdfObject) -> Result<Self> {
        unimplemented!()
    }
}
