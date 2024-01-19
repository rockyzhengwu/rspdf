use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::device::Device;
use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::{create_font, Font};
use crate::geom::rectangle::Rectangle;
use crate::object::{PDFDictionary, PDFObject, PDFStream};
use crate::page::content_interpreter::ContentInterpreter;
use crate::pagetree::PageNodeRef;

pub mod content_interpreter;
pub mod content_parser;
pub mod context;
pub mod graphics_state;
pub mod image;
pub mod operation;
pub mod page_object;
pub mod text;

#[derive(Debug)]
pub struct Page<'a, T: Seek + Read> {
    data: PDFDictionary,
    doc: &'a Document<T>,
    fonts: HashMap<String, Font>,
    resources: PDFDictionary,
}

impl<'a, T: Seek + Read> Page<'a, T> {
    pub fn try_new(node: PageNodeRef, doc: &'a Document<T>) -> PDFResult<Self> {
        let data = node.borrow().data().clone();
        // TODO create a page resource struct ?
        let resources = node.borrow().resources(doc)?;
        let mut fonts: HashMap<String, Font> = HashMap::new();
        if let Some(fd) = resources.get("Font") {
            let fontinfo = match fd {
                PDFObject::Indirect(_) => {
                    let font_dict: PDFDictionary = doc.read_indirect(fd)?.try_into()?;
                    font_dict
                }
                PDFObject::Dictionary(font_dict) => font_dict.to_owned(),
                _ => HashMap::new(),
            };
            for (key, v) in &fontinfo {
                let fontobj = doc.read_indirect(v)?;
                let font = create_font(key, &fontobj, doc)?;
                fonts.insert(key.to_owned(), font);
            }
        }

        Ok(Page {
            data,
            doc,
            fonts,
            resources,
        })
    }

    pub fn get_font(&self, tag: &str) -> PDFResult<&Font> {
        if let Some(font) = self.fonts.get(tag) {
            return Ok(font);
        }
        Err(PDFError::InvalidSyntax(format!(
            "get fonts {:?} not exist in resources",
            self.resources
        )))
    }

    pub fn display(&self, device: Box<dyn Device>) -> PDFResult<()> {
        let mut interpreter = ContentInterpreter::try_new(self, self.doc, device)?;
        interpreter.run()?;
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

    fn resources(&self) -> PDFResult<PDFObject> {
        let resource_ref = self.data.get("Resources").unwrap();
        let resource_obj = self.doc.read_indirect(resource_ref)?;
        Ok(resource_obj)
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
