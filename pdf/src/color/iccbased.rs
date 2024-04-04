use crate::color::{ColorRGBValue, ColorSpace};
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use std::io::Write;

#[derive(Debug, Default, Clone)]
pub struct IccProfile {}

#[derive(Debug, Clone)]
pub struct IccBased {
    n: u8,
    alternate: Option<Box<ColorSpace>>,
    profile: IccProfile,
}

impl IccBased {
    pub fn try_new(obj: &PDFObject) -> PDFResult<Self> {
        let n = obj
            .get_value_as_u8("N")
            .ok_or(PDFError::ColorError("IccBased need N".to_string()))??;
        if !matches!(n, 1 | 3 | 4) {
            return Err(PDFError::ColorError(format!(
                "IccBased Colorspace n need in 1, 3,4 got:{:?}",
                n
            )));
        }
        let mut file = std::fs::File::create("test_color.icc").unwrap();
        file.write_all(obj.bytes().unwrap().as_slice()).unwrap();
        // TODO parse ICCProfile
        Ok(IccBased {
            n,
            alternate: None,
            profile: IccProfile::default(),
        })
    }

    pub fn to_rgb(&self, inputs: &[f32]) -> PDFResult<ColorRGBValue> {
        // TODO not support iccprofile
        match self.n {
            1 => {
                panic!("not implement");
            }
            3 => {
                let r = (inputs.first().unwrap().to_owned() * 255.0) as u32;
                let g = (inputs.get(1).unwrap().to_owned() * 255.0) as u32;
                let b = (inputs.last().unwrap().to_owned() * 255.0) as u32;
                Ok(ColorRGBValue(255-r , 255-g  , 255-b ))
            }
            4 => {
                panic!("not implement");
            }
            _ => {
                panic!("Invald input output number");
            }
        }
    }
}
