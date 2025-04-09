use std::path::Path;

use crate::catalog::Catalog;
use crate::error::{PdfError, Result};
use crate::object::{ObjectId, PdfObject};
use crate::page::Page;
use crate::reader::PdfReader;
use crate::xref::Xref;

pub struct Document {
    xref: Xref,
    catalog: Catalog,
}

impl Document {
    pub fn new_from_file<P: AsRef<Path>>(p: P, password: Option<&[u8]>) -> Result<Self> {
        let reader = PdfReader::new_from_file(p)?;
        let xref = Xref::try_new(reader, password)?;

        let mut doc = Document {
            xref,
            catalog: Catalog::default(),
        };
        doc.load_catalog()?;

        Ok(doc)
    }

    fn load_catalog(&mut self) -> Result<()> {
        let root = self
            .xref
            .trailer()
            .get("Root")
            .ok_or(PdfError::DocumentStructure(
                "Root not in trailer".to_string(),
            ))?;
        let catalog = self.xref.read_object(root)?.to_dict()?;
        self.catalog = Catalog::try_new(catalog, &self.xref)?;
        Ok(())
    }

    pub fn get_page(&self, i: &u32) -> Option<Page> {
        let node = self.catalog.get_page(i).unwrap();
        Some(Page::try_new(node.clone(), &self.xref).unwrap())
    }

    pub fn objects_num(&self) -> usize {
        self.xref.objects_num()
    }

    pub fn total_page(&self) -> Result<u32> {
        self.catalog.total_page()
    }
    pub fn read_object(&self, num: u32, gen: u16) -> Result<PdfObject> {
        self.xref.read_indirect_object(&(num, gen))
    }
}
