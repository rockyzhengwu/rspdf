use std::collections::HashMap;

use crate::object::PDFDictionary;

#[derive(Debug, PartialEq, Eq)]
pub enum EntryType {
    Free,
    Normal,
    Compressed,
}

#[derive(Debug, PartialEq)]
pub struct EntryInfo {
    number: u32,
    gen: u32,
    pos: u64,
    entry_type: EntryType,
}

impl EntryInfo {
    pub fn new(number: u32, gen: u32, pos: u64, entry_type: EntryType) -> Self {
        EntryInfo {
            number,
            gen,
            pos,
            entry_type,
        }
    }

    pub fn pos(&self) -> u64 {
        self.pos
    }
}

#[derive(Default, Debug)]
pub struct CrossRefTable {
    trailer: PDFDictionary,
    entries: HashMap<u32, EntryInfo>,
}

impl CrossRefTable {
    pub fn new(trailer: PDFDictionary, entries: HashMap<u32, EntryInfo>) -> Self {
        CrossRefTable { trailer, entries }
    }
    pub fn set_trailer(&mut self, trailer: PDFDictionary) {
        self.trailer = trailer;
    }

    pub fn add_entries(&mut self, entries: Vec<EntryInfo>) {
        for entry in entries {
            self.entries.insert(entry.number, entry);
        }
    }

    pub fn merge(&mut self, other: CrossRefTable) {
        if self.trailer.is_empty() {
            self.trailer = other.trailer;
        }

        for (key, v) in other.entries {
            self.entries.insert(key, v);
        }
    }

    pub fn trailer(&self) -> &PDFDictionary {
        &self.trailer
    }

    pub fn get_entry(&self, objnum: &u32) -> Option<&EntryInfo> {
        self.entries.get(objnum)
    }
}
