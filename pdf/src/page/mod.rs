use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;

use crate::device::Device;
use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::pdf_font::{load_font, Font};
use crate::geom::rectangle::Rectangle;
use crate::object::{PDFDictionary, PDFObject, PDFStream};
use crate::page::content_interpreter::ContentInterpreter;
use crate::page::object_iterator::ObjectIterator;
use crate::pagetree::PageNodeRef;

pub mod color;
pub mod color_space;
pub mod content_interpreter;
pub mod content_parser;
pub mod context;
pub mod general_state;
pub mod graphics_object;
pub mod graphics_state;
pub mod image;
pub mod object_iterator;
pub mod operation;
pub mod path_state;
pub mod text;
pub mod text_state;

#[derive(Debug)]
pub struct Page<'a, T: Seek + Read> {
    number: u32,
    noderef: PageNodeRef,
    doc: &'a Document<T>,
    resources: PDFDictionary,
    data: PDFDictionary,
}

impl<'a, T: Seek + Read> Page<'a, T> {
    pub fn try_new(number: &u32, node: PageNodeRef, doc: &'a Document<T>) -> PDFResult<Self> {
        let data = node.borrow().data().to_owned();
        let resources = node.borrow().resources(doc)?;

        // TODO create a page resource struct ?
        Ok(Page {
            number: number.to_owned(),
            noderef: node,
            doc,
            resources,
            data,
        })
    }

    pub fn get_font(&self, tag: &str) -> PDFResult<Font> {
        if let Some(font) = self.doc.get_font(tag) {
            return Ok(font);
        }
        if let Some(fd) = self.resources.get("Font") {
            let fontinfo: PDFDictionary = self.doc.get_object_without_indriect(fd)?.try_into()?;

            match fontinfo.get(tag) {
                Some(vv) => {
                    let fontobj = self.doc.read_indirect(vv)?;
                    let font = load_font(&fontobj, self.doc)?;
                    self.doc.add_font(tag, font.clone());
                    return Ok(font);
                }
                None => {
                    return Err(PDFError::InvalidSyntax(format!(
                        "get fonts {:?} not exist in resources:{:?}",
                        tag, fontinfo
                    )));
                }
            }
        }

        Err(PDFError::InvalidSyntax(format!(
            "get fonts {:?} not exist in resources",
            self.resources
        )))
    }

    pub fn grapics_objects(&self) -> PDFResult<ObjectIterator<T>> {
        let mut interpreter = ContentInterpreter::try_new(self, self.doc)?;
        interpreter.start()?;
        let iterator = ObjectIterator::new(interpreter);
        Ok(iterator)
    }

    pub fn display<D: Device>(&self, _device: Rc<RefCell<D>>) -> PDFResult<()> {
        let mut interpreter = ContentInterpreter::try_new(self, self.doc)?;
        interpreter.start()?;
        while let Some(_obj) = interpreter.poll()? {
            // TODO
            // println!("obj:{:?}", obj);
        }
        Ok(())
    }

    fn contents(&self) -> PDFResult<Vec<PDFStream>> {
        let mut content_streams = Vec::new();
        if let Some(contents) = self.data.get("Contents") {
            match contents {
                PDFObject::Indirect(_) => {
                    let cs: PDFObject = self.doc.read_indirect(contents)?;
                    content_streams.push(cs.try_into()?)
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
        self.noderef.borrow().media_bbox()
    }

    pub fn crop_bbox(&self) -> PDFResult<Option<Rectangle>> {
        self.noderef.borrow().crop_bbox()
    }
}
