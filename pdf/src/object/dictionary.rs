use std::collections::HashMap;

use crate::object::PdfObject;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PdfDict {
    entries: HashMap<String, PdfObject>,
}

impl PdfDict {
    pub fn new(entries: HashMap<String, PdfObject>) -> Self {
        Self { entries }
    }

    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.entries.get(key)
    }

    pub fn entries(&self) -> &HashMap<String, PdfObject> {
        &self.entries
    }
}
