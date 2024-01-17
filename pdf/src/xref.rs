use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};

use log::warn;

use crate::errors::{PDFError, PDFResult};
use crate::object::{PDFDictionary, PDFObject};
use crate::reader::Reader;

#[derive(Debug, PartialEq, Eq)]
pub enum XRefEntryType {
    XRefEntryFree,
    XRefEntryUncompressed,
    XRefEntryCompressed,
}

#[derive(Debug, PartialEq)]
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
        self.trailer.get("Info")
    }

    pub fn root(&self) -> PDFResult<&PDFObject> {
        self.trailer
            .get("Root")
            .ok_or(PDFError::InvalidSyntax("Root not in trailer".to_string()))
    }

    pub fn attribute(&self, name: &str) -> Option<&PDFObject> {
        self.trailer.get(name)
    }

    pub fn indirect_entry(&self, indirect: &PDFObject) -> PDFResult<&XRefEntry> {
        match indirect {
            PDFObject::Indirect(ref obj) => {
                let mut entry = self
                    .entries
                    .get(&(obj.number() as i64, obj.gen() as i64))
                    .unwrap();
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

    pub fn fetch_object(&self, indirect: &PDFObject) -> PDFResult<PDFObject> {
        let entry = self.indirect_entry(indirect)?;
        let mut obj = self.reader.borrow_mut().fetch_object(entry)?;
        match obj {
            PDFObject::Stream(ref mut s) => {
                let lobj = s.length();
                let length = match lobj {
                    Some(&PDFObject::Indirect(_)) => {
                        let le = self.indirect_entry(lobj.unwrap())?;
                        Some(self.reader.borrow_mut().fetch_object(le)?.as_i64()?)
                    }
                    Some(PDFObject::Number(ref v)) => Some(v.as_i64()),
                    _ => None,
                };
                let buffer = match length {
                    Some(l) => self
                        .reader
                        .borrow_mut()
                        .read_stream_content(s, l as usize)?,
                    None => {
                        warn!("Invalid Length in Stream :{:?}", indirect);
                        self.reader.borrow_mut().read_stream_content_unitl_end(s)?
                    }
                };
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

    // just use in test now
    pub fn entries(&self) -> &XRefEntryTable {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::XRef;
    use crate::object::{PDFIndirect, PDFObject};
    use crate::reader::Reader;

    use std::cell::RefCell;
    use std::fs::File;
    use std::path::PathBuf;

    fn abslute_path(name: &str) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push(format!("tests/resources/{}", name));
        d
    }
    fn create_xref(path: PathBuf) -> XRef<File> {
        let file = File::open(path).unwrap();
        let mut reader = Reader::new(file);
        let (trailer, entries) = reader.read_xref().unwrap();
        XRef::try_new(RefCell::new(reader), trailer, entries).unwrap()
    }

    #[test]
    fn test_stream_without_length() {
        let path = abslute_path("stream_without_length.pdf");
        let xref = create_xref(path);
        let stream = xref
            .fetch_object(&PDFObject::Indirect(PDFIndirect::new(6, 0)))
            .unwrap();
        assert_eq!(stream.bytes().unwrap().len(), 83);
    }
}
