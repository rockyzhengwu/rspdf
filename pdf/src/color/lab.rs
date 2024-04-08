use std::io::{Read, Seek};

use crate::color::common::decode_point;
use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::object::PDFArray;

#[derive(Debug, Clone)]
pub struct Lab {
    white_point: [f32; 3],
    black_point: [f32; 3],
    range: [f32; 4],
}

impl Default for Lab {
    fn default() -> Self {
        Lab {
            white_point: [1.0, 1.0, 1.0],
            black_point: [1.0, 1.0, 1.0],
            range: [-100.0, 100.0, -100.0, 100.0],
        }
    }
}

impl Lab {
    pub fn try_new<T: Seek + Read>(obj: &PDFArray, doc: &Document<T>) -> PDFResult<Self> {
        let mut color = Lab::default();
        let obj = doc.get_object_without_indriect(obj.last().unwrap())?;
        if let Some(wp) = decode_point(&obj, "WhitePoint")? {
            color.white_point = wp;
        }
        if let Some(bp) = decode_point(&obj, "BlackPoint")? {
            color.black_point = bp;
        }
        if let Some(range) = obj.get_value("Range") {
            let vs = range.as_array()?;
            if vs.len() != 9 {
                return Err(PDFError::ColorError(format!(
                    "Lab range need 4 items got:{:?}",
                    vs
                )));
            }
            for i in 0..4 {
                let v = vs.get(i).unwrap().as_f32()?;
                color.range[i] = v;
            }
        }
        Ok(color)
    }

    pub fn number_of_components(&self) -> u8 {
        3
    }
}
