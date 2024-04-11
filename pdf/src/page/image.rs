use crate::color::{ColorRGBValue, ColorSpace};
use crate::errors::{PDFError, PDFResult};
use crate::geom::matrix::Matrix;
use crate::object::PDFObject;
use crate::page::graphics_state::GraphicsState;

#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct Image {
    obj: PDFObject,
    colorspace: Option<ColorSpace>,
    graphics_state: GraphicsState,
    is_inline: bool,
}

impl Image {
    pub fn new(
        obj: PDFObject,
        colorspace: Option<ColorSpace>,
        graphics_state: GraphicsState,
        is_inline: bool,
    ) -> Self {
        Image {
            obj,
            colorspace,
            graphics_state,
            is_inline,
        }
    }
    pub fn ctm(&self) -> &Matrix {
        self.graphics_state.ctm()
    }

    pub fn width(&self) -> PDFResult<f64> {
        self.obj
            .get_value("Width")
            .ok_or(PDFError::PageError("Width not in image object".to_string()))?
            .as_f64()
    }

    pub fn height(&self) -> PDFResult<f64> {
        self.obj
            .get_value("Height")
            .ok_or(PDFError::PageError(
                "Height not in Image object".to_string(),
            ))?
            .as_f64()
    }

    pub fn data(&self) -> PDFResult<Vec<u8>> {
        self.obj.bytes()
    }

    pub fn bits_per_component(&self) -> PDFResult<u8> {
        self.obj
            .get_value("BitsPerComponent")
            .ok_or(PDFError::PageError(
                "BitsPerComponent not in Image object".to_string(),
            ))?
            .as_u8()
    }

    pub fn colorsapce(&self) -> Option<&ColorSpace> {
        self.colorspace.as_ref()
    }

    pub fn rgb_image(&self) -> PDFResult<Vec<ColorRGBValue>> {
        let bytes = self.obj.bytes()?;
        self.colorspace
            .as_ref()
            .unwrap()
            .to_rgb_image(bytes.as_slice())
    }
}
