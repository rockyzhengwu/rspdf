use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::object::PDFDictionary;
use crate::pagetree::{PageNodeRef, PageTree};

#[derive(Default, Debug)]
pub enum PageLayout {
    #[default]
    SinglePage,
    OneColumn,
    TwoColumnLeft,
    TwoColumnRight,
    TwoPageLeft,
    TwoPageRight,
}
impl PageLayout {
    pub fn new(pagelayout: &str) -> Self {
        match pagelayout {
            "SinglePage" => PageLayout::SinglePage,
            "OneColumn" => PageLayout::OneColumn,
            "TwoColumnLeft" => PageLayout::TwoColumnLeft,
            "TwoColumnRight" => PageLayout::TwoColumnRight,
            "TwoPageLeft" => PageLayout::TwoPageLeft,
            "TwoPageRight" => PageLayout::TwoPageRight,
            _ => PageLayout::SinglePage,
        }
    }
}

#[derive(Default, Debug)]
pub struct Catalog {
    page_layout: PageLayout,
    outlines: Option<PDFDictionary>,
    page_tree: PageTree,
}

impl Catalog {
    pub fn try_new<T: Seek + Read>(root: PDFDictionary, doc: &Document<T>) -> PDFResult<Self> {
        let mut catalog = Catalog::default();
        if let Some(pl) = root.get("PageLayout") {
            catalog.page_layout = PageLayout::new(&pl.as_string()?);
        }

        if let Some(ot) = root.get("Outlines") {
            let outlines: PDFDictionary = doc.read_indirect(ot)?.try_into()?;
            catalog.outlines = Some(outlines);
        }
        let pagetree = PageTree::try_new(root, doc)?;
        catalog.page_tree = pagetree;
        Ok(catalog)
    }

    pub fn get_page(&self, pageindex: &u32) -> Option<&PageNodeRef> {
        self.page_tree.get_page(pageindex)
    }

    pub fn page_count(&self) -> PDFResult<u32> {
        self.page_tree.count()
    }
}
