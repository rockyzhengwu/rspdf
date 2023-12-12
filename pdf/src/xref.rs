use std::collections::HashMap;

use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFDictionary, PDFName, PDFObject};

#[derive(Debug)]
pub enum XRefEntryType {
    XRefEntryFree,
    XRefEntryUncompressed,
    XRefEntryCompressed,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct XRefEntry {
    number: i64,
    offset: i64,
    gen: i64,
    stream_offset: i64,
    xtype: XRefEntryType,
}

pub type XRefEntryTable = HashMap<(i64, i64), XRefEntry>;

impl XRefEntry {
    pub fn new(number: i64, offset: i64, gen: i64, xtype: XRefEntryType) -> Self {
        XRefEntry {
            number,
            offset,
            gen,
            xtype,
            stream_offset: 0,
        }
    }
    pub fn set_stream_offset(&mut self, stream_offset: i64) {
        self.stream_offset = stream_offset
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }
    pub fn number(&self) -> i64 {
        self.number
    }

    pub fn gen(&self) -> i64 {
        self.gen
    }
}

#[derive(Debug)]
pub struct XRef {
    entries: HashMap<(i64, i64), XRefEntry>,
    trailer: PDFDictionary,
}

impl XRef {
    pub fn try_new(trailer: PDFObject, entries: HashMap<(i64, i64), XRefEntry>) -> PDFResult<Self> {
        Ok(XRef {
            entries,
            trailer: trailer.try_into()?,
        })
    }

    pub fn set_trainer(&mut self, trailer: PDFDictionary) {
        if self.trailer.is_empty() {
            self.trailer = trailer;
        }
    }

    pub fn info(&self) -> Option<&PDFObject> {
        self.trailer.get(&PDFName::new("Info"))
    }

    pub fn root(&self) -> PDFResult<&PDFObject> {
        self.trailer
            .get(&PDFName::new("Root"))
            .ok_or(PDFError::InvalidSyntax("Root not in trailer".to_string()))
    }

    pub fn attribute(&self, name: &str) -> Option<&PDFObject> {
        self.trailer.get(&PDFName::new(name))
    }

    pub fn indirect_entry(&self, indirect: &PDFObject) -> PDFResult<&XRefEntry> {
        match indirect {
            PDFObject::Indirect(ref obj) => {
                let mut entry = self.entries.get(&(obj.number(), obj.gen())).unwrap();
                while entry.stream_offset != 0 {
                    entry = self.entries.get(&(entry.stream_offset, 0)).unwrap();
                }
                Ok(entry)
            }
            _ => Err(PDFError::InvalidSyntax(format!(
                "{:?} not a PDFIndirect",
                indirect
            ))),
        }
    }

    // TODO delete this
    pub fn get(&self, name: &str) -> PDFResult<&PDFObject> {
        self.trailer
            .get(&PDFName::new(name))
            .ok_or(PDFError::InvalidSyntax(format!("{} not in Xref", name)))
    }
}
