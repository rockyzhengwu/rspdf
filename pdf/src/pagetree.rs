use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::{Rc, Weak};

use crate::error::{PdfError, Result};
use crate::geom::rect::Rect;
use crate::object::dictionary::PdfDict;
use crate::object::PdfObject;
use crate::xref::Xref;

#[derive(Debug, Clone, Default)]
enum PageNodeType {
    #[default]
    Intermediate,
    Leaf,
}

pub type PageNodeRef = Rc<RefCell<PageNode>>;

#[derive(Debug, Default)]
pub struct PageNode {
    node_type: PageNodeType,
    count: u32,
    parent: Option<Weak<RefCell<PageNode>>>,
    kids: Vec<PageNodeRef>,
    dict: PdfDict,
    index: u32,
}

impl PageNode {
    pub fn new(dict: PdfDict, parent: Option<Weak<RefCell<PageNode>>>) -> Self {
        let node_type = if dict.get("Kids").is_some() {
            PageNodeType::Intermediate
        } else {
            PageNodeType::Leaf
        };
        let count = match dict.get("Count") {
            Some(v) => v.integer().unwrap_or_default(),
            _ => 0,
        };
        PageNode {
            node_type,
            parent,
            dict,
            count: count as u32,
            kids: Vec::new(),
            index: 0,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn add_kid(&mut self, child: PageNodeRef) {
        self.kids.push(child)
    }

    pub fn count(&self) -> &u32 {
        &self.count
    }

    pub fn dict(&self) -> &PdfDict {
        &self.dict
    }

    pub fn kids(&self) -> &[PageNodeRef] {
        self.kids.as_slice()
    }

    pub fn resources(&self, xref: &Xref) -> Result<PdfDict> {
        match self.dict().get("Resources") {
            Some(res) => match res {
                PdfObject::Indirect(_) => {
                    let obj = xref.read_object(res)?.as_dict()?.to_owned();
                    Ok(obj)
                }
                PdfObject::Dict(obj) => Ok(obj.to_owned()),
                _ => Err(PdfError::Page(format!(
                    "resource not a Dictionary obj:{:?}",
                    res
                ))),
            },
            None => match self.parent {
                Some(ref p) => p.upgrade().unwrap().borrow().resources(xref),
                None => Err(PdfError::Page("Page has no resource".to_string())),
            },
        }
    }
    pub fn mediabox(&self) -> Result<Option<Rect>> {
        match self.dict.get("MediaBox") {
            Some(o) => {
                let bbox = o
                    .as_array()
                    .map_err(|_| PdfError::Page("MediaBox is not an array".to_string()))?;
                let rect = Rect::new_from_pdf_bbox(bbox)
                    .map_err(|e| PdfError::Page(format!("create Page mediabox error:{:?}", e)))?;
                Ok(Some(rect))
            }
            None => match self.parent {
                Some(ref p) => p.upgrade().unwrap().borrow().mediabox(),
                None => Ok(None),
            },
        }
    }

    pub fn cropbox(&self) -> Result<Option<Rect>> {
        match self.dict.get("CropBox") {
            Some(o) => {
                let bbox = o
                    .as_array()
                    .map_err(|_| PdfError::Page("CropBox is not an array".to_string()))?;
                let rect = Rect::new_from_pdf_bbox(bbox)
                    .map_err(|e| PdfError::Page(format!("create Page CropBox error:{:?}", e)))?;
                Ok(Some(rect))
            }
            None => match self.parent {
                Some(ref p) => p.upgrade().unwrap().borrow().cropbox(),
                None => Ok(None),
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct PageTree {
    root: PageNodeRef,
    pages: HashMap<u32, PageNodeRef>,
}

impl PageTree {
    pub fn try_new(catalog: PdfDict, xref: &Xref) -> Result<Self> {
        if let Some(pagesref) = catalog.get("Pages") {
            let pages = xref.read_object(pagesref)?;
            let root = create_pagetree(pages.as_dict()?.to_owned(), xref, None)?;
            let pages = create_pages(root.clone());

            Ok(PageTree { root, pages })
        } else {
            let root = Rc::new(RefCell::new(PageNode::default()));
            Ok(PageTree {
                root,
                pages: HashMap::new(),
            })
        }
    }

    pub fn get_page(&self, index: &u32) -> Option<&PageNodeRef> {
        self.pages.get(index)
    }

    pub fn count(&self) -> Result<u32> {
        let count = self.root.borrow().dict().get("Count").unwrap().integer()? as u32;
        Ok(count)
    }
}

fn create_pages(root: PageNodeRef) -> HashMap<u32, PageNodeRef> {
    let mut res = HashMap::new();
    let mut index = 0;
    let mut queue: VecDeque<PageNodeRef> = VecDeque::new();
    queue.push_back(root);
    while !queue.is_empty() {
        if let Some(node) = queue.pop_front() {
            match node.as_ref().borrow().node_type {
                PageNodeType::Intermediate => {
                    for kid in node.as_ref().borrow().kids() {
                        queue.push_back(kid.clone());
                    }
                }
                PageNodeType::Leaf => {
                    res.insert(index, node.clone());
                    index += 1;
                }
            }
        }
    }
    res
}

fn create_pagetree(
    root: PdfDict,
    xref: &Xref,
    parent: Option<Weak<RefCell<PageNode>>>,
) -> Result<PageNodeRef> {
    let node = PageNode::new(root.clone(), parent);
    let noderef = Rc::new(RefCell::new(node));
    if let Some(PdfObject::Array(kids)) = root.get("Kids") {
        for kid in kids.iter() {
            let kid_data = xref.read_object(kid)?;
            let child = create_pagetree(
                kid_data.as_dict()?.to_owned(),
                xref,
                Some(Rc::downgrade(&noderef)),
            )?;
            noderef.borrow_mut().add_kid(child);
        }
    }
    Ok(noderef)
}
