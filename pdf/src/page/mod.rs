use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;

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
    number: u32,
    data: PDFDictionary,
    doc: &'a Document<T>,
    resources: PDFDictionary,
}

impl<'a, T: Seek + Read> Page<'a, T> {
    pub fn try_new(number: &u32, node: PageNodeRef, doc: &'a Document<T>) -> PDFResult<Self> {
        let data = node.borrow().data().clone();
        let resources = node.borrow().resources(doc)?;
        // TODO create a page resource struct ?
        Ok(Page {
            number:number.to_owned(),
            data,
            doc,
            resources,
        })
    }

    pub fn get_font(&self, tag: &str) -> PDFResult<Rc<Font>> {
        if let Some(font) = self.doc.get_font(tag) {
            return Ok(font);
        }
        if let Some(fd) = self.resources.get("Font") {
            let fontinfo = match fd {
                PDFObject::Indirect(_) => {
                    let font_dict: PDFDictionary = self.doc.read_indirect(fd)?.try_into()?;
                    font_dict
                }
                PDFObject::Dictionary(font_dict) => font_dict.to_owned(),
                _ => HashMap::new(),
            };
            match fontinfo.get(tag) {
                Some(vv) => {
                    let fontobj = self.doc.read_indirect(vv)?;
                    let font = Rc::new(create_font(tag, &fontobj, self.doc)?);
                    self.doc.add_font(tag, font.clone());
                    return Ok(font);
                }
                None => {
                    return Err(PDFError::InvalidSyntax(format!(
                        "get fonts {:?} not exist in resources",
                        self.resources
                    )));
                }
            }
        }

        Err(PDFError::InvalidSyntax(format!(
            "get fonts {:?} not exist in resources",
            self.resources
        )))
    }

    pub fn display<D: Device>(&self, device: Rc<RefCell<D>>) -> PDFResult<()> {
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

    pub fn resources(&self) -> PDFResult<PDFObject> {
        match self.data.get("Resources") {
            Some(obj) => match obj {
                PDFObject::Dictionary(_) => Ok(obj.to_owned()),
                PDFObject::Indirect(_) => self.doc.read_indirect(obj),
                _ => Err(PDFError::InvalidSyntax(format!(
                    "resource obj type error:{:?}",
                    obj
                ))),
            },
            None => Ok(PDFObject::Dictionary(HashMap::new())),
        }
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
