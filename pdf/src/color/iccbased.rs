use std::io::{Read, Seek};

use crate::color::{create_colorspace, ColorSpace};
use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

use super::{CMYKValue, ColorValue, GrayValue, RGBValue};

#[derive(Debug, Default, Clone)]
pub struct IccProfile {}

#[derive(Debug, Clone)]
pub struct IccBased {
    n: u8,
    alternate: Option<Box<ColorSpace>>,
    profile: IccProfile,
}

impl IccBased {
    pub fn try_new<T: Seek + Read>(obj: &PDFObject, doc: &Document<T>) -> PDFResult<Self> {
        let n = obj
            .get_value_as_u8("N")
            .ok_or(PDFError::ColorError("IccBased need N".to_string()))??;
        if !matches!(n, 1 | 3 | 4) {
            return Err(PDFError::ColorError(format!(
                "IccBased Colorspace n need in 1, 3,4 got:{:?}",
                n
            )));
        }
        let alternate = obj.get_value("Alternate").unwrap();
        let alternate = doc.get_object_without_indriect(alternate)?;
        let alternate_space = create_colorspace(&alternate, doc)?;
        // TODO parse ICCProfile
        Ok(IccBased {
            n,
            alternate: Some(Box::new(alternate_space)),
            profile: IccProfile::default(),
        })
    }

    pub fn to_rgb(&self, inputs: &[f32]) -> PDFResult<RGBValue> {
        // TODO not support iccprofile
        match self.n {
            1 => {
                panic!("not implement");
            }
            3 => {
                let r = (inputs.first().unwrap().to_owned() * 255.0) as u8;
                let g = (inputs.get(1).unwrap().to_owned() * 255.0) as u8;
                let b = (inputs.last().unwrap().to_owned() * 255.0) as u8;
                Ok(RGBValue(r, g, b))
            }
            4 => {
                panic!("not implement");
            }
            _ => {
                panic!("Invald input output number");
            }
        }
    }

    pub fn number_of_components(&self) -> u8 {
        self.n
    }

    pub fn to_rgb_image(&self, bytes: &[u8]) -> PDFResult<Vec<RGBValue>> {
        let mut image = Vec::new();
        for chunk in bytes.chunks(3) {
            let inputs: Vec<f32> = chunk
                .to_owned()
                .iter()
                .map(|x| (x.to_owned() as f32) / 255.0)
                .collect();
            let rgb = self.to_rgb(inputs.as_slice())?;
            image.push(rgb)
        }
        Ok(image)
    }
}
