use crate::error::Result;
use crate::object::dictionary::PdfDict;
use crate::pagetree::{PageNodeRef, PageTree};
use crate::xref::Xref;

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
    outlines: Option<PdfDict>,
    page_tree: PageTree,
}

impl Catalog {
    pub fn try_new(root: PdfDict, xref: &Xref) -> Result<Self> {
        let mut catalog = Catalog::default();
        if let Some(pl) = root.get("PageLayout") {
            catalog.page_layout = PageLayout::new(pl.as_name()?.name());
        }
        if let Some(ot) = root.get("Outlines") {
            let outlines: PdfDict = xref.read_object(ot)?.as_dict()?.to_owned();
            catalog.outlines = Some(outlines);
        }

        let pagetree = PageTree::try_new(root, xref)?;
        catalog.page_tree = pagetree;
        Ok(catalog)
    }

    pub fn get_page(&self, pageindex: &u32) -> Option<&PageNodeRef> {
        self.page_tree.get_page(pageindex)
    }

    pub fn total_page(&self) -> Result<u32> {
        self.page_tree.count()
    }
}
