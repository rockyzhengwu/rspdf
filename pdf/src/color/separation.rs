use std::io::{Read, Seek};

use crate::color::careate_colorspace;
use crate::color::ColorSpace;
use crate::document::Document;
use crate::errors::PDFResult;
use crate::object::{PDFArray, PDFObject};
use crate::page::function::simple::SimpleFunction;

#[derive(Debug, Clone)]
pub struct Separation {
    alternate_space: Box<ColorSpace>,
    tint_transform: SimpleFunction,
}

impl Separation {
    pub fn try_new<T: Seek + Read>(arr: &PDFArray, doc: &Document<T>) -> PDFResult<Self> {
        let name = arr.get(1).unwrap();
        println!("name: {:?}", name);
        let alternate_space = doc
            .get_object_without_indriect(arr.get(2).unwrap())
            .unwrap();
        let alternate_space = careate_colorspace(&alternate_space, doc)?;
        let tint_transform = doc
            .get_object_without_indriect(arr.get(3).unwrap())
            .unwrap();
        let transform = SimpleFunction::try_new(&tint_transform)?;
        Ok(Separation {
            alternate_space: Box::new(alternate_space),
            tint_transform: transform,
        })
    }
}
