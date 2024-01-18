use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::geom::rectangle::Rectangle;
use crate::object::{PDFDictionary, PDFObject, PDFStream};
use crate::page::content_parser::ContentParser;

#[derive(Debug)]
pub struct Page<'a, T: Seek + Read> {
    data: PDFDictionary,
    doc: &'a Document<T>,
}

pub mod content_interpreter;
pub mod content_parser;
pub mod operation;

impl<'a, T: Seek + Read> Page<'a, T> {
    pub fn new(pagedict: PDFDictionary, doc: &'a Document<T>) -> Self {
        Page {
            data: pagedict,
            doc,
        }
    }

    pub fn objects(&self) -> PDFResult<()> {
        let contents = self.contents()?;
        let mut buffers = Vec::new();
        for content in contents {
            buffers.extend(content.bytes());
        }
        let mut parser = ContentParser::try_new(buffers)?;
        loop {
            let op = parser.parse_operation().unwrap();
            println!("{:?}", op);
        }

        Ok(())
    }

    fn contents(&self) -> PDFResult<Vec<PDFStream>> {
        let mut content_streams = Vec::new();
        if let Some(contents) = self.data.get("Contents") {
            match contents {
                PDFObject::Indirect(_) => {
                    let cs: PDFStream = self.doc.read_indirect(contents)?.try_into()?;
                    content_streams.push(cs)
                }
                PDFObject::Arrray(vals) => {
                    for ci in vals {
                        let cs: PDFStream = self.doc.read_indirect(ci)?.try_into()?;
                        content_streams.push(cs)
                    }
                }
                _ => {
                    return Err(PDFError::InvalidContentSyntax(format!(
                        "content need a stream or array got:{:?}",
                        contents
                    )))
                }
            }
        }
        Ok(content_streams)
    }

    fn resources(&self) {
        let resource_ref = self.data.get("Resources").unwrap();
        let resource_obj = self.doc.read_indirect(resource_ref);
        println!("{:?}", resource_obj);
    }

    pub fn media_bbox(&self) -> PDFResult<Option<Rectangle>> {
        match self.data.get("MediaBox") {
            Some(PDFObject::Arrray(arrs)) => {
                let lx = arrs[0].as_f64()?;
                let ly = arrs[1].as_f64()?;
                let ux = arrs[2].as_f64()?;
                let uy = arrs[3].as_f64()?;
                Ok(Some(Rectangle::new(lx, ly, ux, uy)))
            }
            Some(obj) => Err(PDFError::InvalidContentSyntax(format!(
                "page mediabox not a rectanble,{:?}",
                obj
            ))),
            None => Ok(None),
        }
    }

    pub fn crop_bbox(&self) -> PDFResult<Option<Rectangle>> {
        match self.data.get("CropBox") {
            Some(PDFObject::Arrray(arrs)) => {
                let lx = arrs[0].as_f64()?;
                let ly = arrs[1].as_f64()?;
                let ux = arrs[2].as_f64()?;
                let uy = arrs[3].as_f64()?;
                Ok(Some(Rectangle::new(lx, ly, ux, uy)))
            }
            Some(obj) => Err(PDFError::InvalidContentSyntax(format!(
                "page cropbox not a rectanble,{:?}",
                obj
            ))),
            None => Ok(None),
        }
    }
}
