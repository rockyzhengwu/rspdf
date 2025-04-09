use crate::{
    color::{parse_colorspace, value::ColorRgb},
    error::{PdfError, Result},
    function::{create_function, Function},
    object::array::PdfArray,
    xref::Xref,
};

use super::{value::ColorValue, ColorSpace};

#[derive(Debug, Clone)]
pub struct Separation {
    name: String,
    alternate_space: Box<ColorSpace>,
    tint_transform: Function,
}

impl Separation {
    pub fn try_new(arr: &PdfArray, xref: &Xref) -> Result<Self> {
        let name = arr
            .get(1)
            .ok_or(PdfError::Color(
                "Separation Colorspace name is Nonne".to_string(),
            ))?
            .as_name()
            .map_err(|_| {
                PdfError::Color("Separation colorspace Name is not a pdfName object".to_string())
            })?
            .name();
        let alternate_space = arr.get(2).ok_or(PdfError::Color(
            "Separation ColorSpace alternate_space is None".to_string(),
        ))?;
        let alternate = parse_colorspace(alternate_space, xref)?;
        let tint_transform = arr.get(3).ok_or(PdfError::Color(
            "Separation colorspace initransform is None".to_string(),
        ))?;
        let ts = xref.read_object(tint_transform)?;
        let tint_transform = create_function(&ts, xref)?;
        Ok(Self {
            name: name.to_string(),
            alternate_space: Box::new(alternate),
            tint_transform,
        })
    }

    pub fn default_value(&self) -> ColorValue {
        return ColorValue::new(vec![1.0]);
    }

    pub fn rgb(&self, value: &ColorValue) -> Result<ColorRgb> {
        let values = self.tint_transform.eval(value.values())?;
        self.alternate_space.rgb(&ColorValue::new(values))
    }

    pub fn number_of_components(&self) -> usize {
        1
    }
}
