use std::io::{Read, Seek};

use crate::color::ColorSpace;
use crate::color::{create_colorspace, ColorRGBValue};
use crate::document::Document;
use crate::errors::PDFResult;
use crate::object::PDFArray;
use crate::page::function::PDFFunction;

#[derive(Debug, Clone)]
pub struct Separation {
    alternate_space: Box<ColorSpace>,
    tint_transform: PDFFunction,
}

impl Separation {
    pub fn try_new<T: Seek + Read>(arr: &PDFArray, doc: &Document<T>) -> PDFResult<Self> {
        let name = arr.get(1).unwrap();
        let alternate_space = doc
            .get_object_without_indriect(arr.get(2).unwrap())
            .unwrap();
        let alternate_space = create_colorspace(&alternate_space, doc)?;
        let tint_transform = doc
            .get_object_without_indriect(arr.get(3).unwrap())
            .unwrap();
        let transform = PDFFunction::try_new(&tint_transform)?;
        Ok(Separation {
            alternate_space: Box::new(alternate_space),
            tint_transform: transform,
        })
    }

    pub fn to_rgb(&self, inputs: &[f32]) -> PDFResult<ColorRGBValue> {
        let alter_color = self.tint_transform.eval(inputs)?;
        self.alternate_space.to_rgb(alter_color.as_slice())
    }
}
