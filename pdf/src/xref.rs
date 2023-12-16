use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFDictionary, PDFName, PDFObject};
use crate::reader::Reader;

#[derive(Debug)]
pub enum XRefEntryType {
    XRefEntryFree,
    XRefEntryUncompressed,
    XRefEntryCompressed,
}

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

    pub fn is_free(&self) -> bool {
        matches!(self.xtype, XRefEntryType::XRefEntryFree)
    }
}

pub struct XRef<T: Seek + Read> {
    reader: RefCell<Reader<T>>,
    entries: HashMap<(i64, i64), XRefEntry>,
    trailer: PDFDictionary,
}

impl<T: Seek + Read> XRef<T> {
    pub fn try_new(
        reader: RefCell<Reader<T>>,
        trailer: PDFObject,
        entries: HashMap<(i64, i64), XRefEntry>,
    ) -> PDFResult<Self> {
        Ok(XRef {
            reader,
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

    // TODO simpleile
    pub fn fetch_object(&self, indirect: &PDFObject) -> PDFResult<PDFObject> {
        let entry = self.indirect_entry(indirect)?;
        let mut obj = self.reader.borrow_mut().fetch_object(entry)?;
        match obj {
            PDFObject::Stream(ref mut s) => {
                let lobj = s.length().unwrap();
                let length = match lobj {
                    PDFObject::Indirect(_) => {
                        let le = self.indirect_entry(lobj).unwrap();
                        self.reader.borrow_mut().fetch_object(le)?.as_i64()?
                    }
                    PDFObject::Number(v) => v.as_i64(),
                    _ => {
                        return Err(PDFError::InvalidSyntax(format!(
                            "Length in stream is not Number or Indirect got:{:?}",
                            lobj
                        )));
                    }
                };
                let buffer = self
                    .reader
                    .borrow_mut()
                    .read_stream_content(s, length as usize)?;
                s.set_buffer(buffer);
                Ok(obj)
            }
            _ => Ok(obj),
        }
    }

    pub fn catalog(&self) -> PDFObject {
        let root = self.root().unwrap();
        match root {
            PDFObject::Indirect(_) => self.fetch_object(root).unwrap(),
            PDFObject::Dictionary(_) => root.to_owned(),
            _ => panic!("Root not Indirect or Dictionary"),
        }
    }
}
