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
}

impl Image {
    pub fn new(
        obj: PDFObject,
        colorspace: Option<ColorSpace>,
        graphics_state: GraphicsState,
    ) -> Self {
        Image {
            obj,
            colorspace,
            graphics_state,
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
        let mut image = Vec::new();
        // TODO fix this
        match self.colorspace {
            Some(ColorSpace::Separation(ref sc)) => {
                for b in bytes {
                    let p = (b as f32) / 255.0;
                    let inputs = vec![p];
                    let rgb = sc.to_rgb(inputs.as_slice())?;
                    image.push(rgb);
                }
            }
            Some(ColorSpace::ICCBased(ref sc)) => {
                for chunk in bytes.chunks(3) {
                    let inputs: Vec<f32> = chunk
                        .to_owned()
                        .iter()
                        .map(|x| (x.to_owned() as f32) / 255.0)
                        .collect();
                    let rgb = sc.to_rgb(inputs.as_slice())?;
                    image.push(rgb)
                }
            }
            Some(ColorSpace::DeviceRGB(ref sc)) => {
                for chunk in bytes.chunks(3) {
                    let inputs: Vec<f32> = chunk
                        .to_owned()
                        .iter()
                        .map(|x| (x.to_owned() as f32))
                        .collect();
                    let rgb = sc.to_rgb(inputs.as_slice())?;
                    println!("{:?}", rgb);
                    image.push(rgb)
                }
            }
            Some(ColorSpace::DeviceGray(ref sc)) => {
                println!("gray: {:?}", self.bits_per_component());
            }
            None => {
                println!("colorspace is NOne")
            }
            _ => {
                println!("others{:?}", self.colorspace);
            }
        }
        //pass
        Ok(image)
    }
}
