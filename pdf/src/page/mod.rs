use crate::{
    device::Device,
    error::{PdfError, Result},
    geom::rect::Rect,
    object::{stream::PdfStream, PdfObject},
    page::interpreter::Interpreter,
    pagetree::PageNodeRef,
    xref::Xref,
};

mod content_parser;
pub mod graphics_state;
mod interpreter;
mod operator;
mod resource;

pub mod context;
pub mod image;

use resource::Resources;

pub struct Page<'a> {
    xref: &'a Xref,
    node: PageNodeRef,
    resources: resource::Resources,
}

impl<'a> Page<'a> {
    pub fn try_new(node: PageNodeRef, xref: &'a Xref) -> Result<Self> {
        let res_dict = node.borrow().resources(xref)?;
        let resources = resource::Resources::try_new(&res_dict, xref)?;
        Ok(Self {
            xref,
            node,
            resources,
        })
    }
    pub fn rotated(&self) -> Result<i32> {
        if let Some(v) = self.node.borrow().dict().get("Rotate") {
            return Ok(v.as_number()?.integer());
        }
        return Ok(0);
    }
    pub fn mediabox(&self) -> Result<Rect> {
        // TODO inheritable
        if let Some(rec) = self.node.borrow().mediabox()? {
            return Ok(rec);
        } else {
            return Err(PdfError::Page("Page Mediabox is None".to_string()));
        }
    }

    pub fn cropbox(&self) -> Result<Option<Rect>> {
        if let Some(rec) = self.node.borrow().cropbox()? {
            Ok(Some(rec))
        } else {
            Ok(None)
        }
    }

    pub fn display(&self, p: u32, device: &mut dyn Device) -> Result<()> {
        let mut interpreter = Interpreter::try_new(self, &self.xref)?;
        interpreter.run(p, device).unwrap();
        Ok(())
    }

    pub fn content_stream(&self) -> Result<Vec<PdfStream>> {
        let mut content_streams = Vec::new();
        if let Some(contents) = self.node.borrow().dict().get("Contents") {
            let contents = self.xref.read_object(contents)?;
            match contents {
                PdfObject::Array(arr) => {
                    for ci in arr.iter() {
                        let cs: PdfStream = self.xref.read_object(ci)?.to_stream()?;
                        content_streams.push(cs)
                    }
                }
                PdfObject::Stream(s) => {
                    content_streams.push(s);
                }
                _ => {
                    return Err(PdfError::Page(format!(
                        "content need a stream or array got:{:?}",
                        contents
                    )))
                }
            }
        }
        Ok(content_streams)
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn index(&self) -> u32 {
        self.node.borrow().index()
    }
    pub fn user_unit(&self) -> Result<f32> {
        if let Some(o) = self.node.borrow().dict().get("UserUnit") {
            Ok(o.as_number()?.real())
        } else {
            Ok(1.0)
        }
    }
}
