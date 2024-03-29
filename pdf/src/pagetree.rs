use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{Read, Seek};
use std::rc::{Rc, Weak};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::geom::rectangle::Rectangle;
use crate::object::{PDFDictionary, PDFObject};

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
    data: PDFDictionary,
}

impl PageNode {
    pub fn new(data: PDFDictionary, parent: Option<Weak<RefCell<PageNode>>>) -> Self {
        let node_type = if data.contains_key("Kids") {
            PageNodeType::Intermediate
        } else {
            PageNodeType::Leaf
        };
        let count = match data.get("Count") {
            Some(v) => v.as_u32().unwrap_or_default(),
            _ => 0,
        };
        PageNode {
            node_type,
            parent,
            data,
            count,
            kids: Vec::new(),
        }
    }

    pub fn add_kid(&mut self, child: PageNodeRef) {
        self.kids.push(child)
    }

    pub fn count(&self) -> &u32 {
        &self.count
    }

    pub fn data(&self) -> &PDFDictionary {
        &self.data
    }

    pub fn kids(&self) -> &[PageNodeRef] {
        self.kids.as_slice()
    }

    pub fn resources<T: Seek + Read>(&self, doc: &Document<T>) -> PDFResult<PDFDictionary> {
        match self.data().get("Resources") {
            Some(res) => match res {
                PDFObject::Indirect(_) => {
                    let obj: PDFDictionary = doc.read_indirect(res)?.try_into()?;
                    Ok(obj)
                }
                PDFObject::Dictionary(obj) => Ok(obj.to_owned()),
                _ => Err(PDFError::InvalidSyntax(format!(
                    "resource not a Dictionary obj:{:?}",
                    res
                ))),
            },
            None => match self.parent {
                Some(ref p) => p.upgrade().unwrap().borrow().resources(doc),
                None => Ok(PDFDictionary::default()),
            },
        }
    }

    pub fn media_bbox(&self) -> PDFResult<Option<Rectangle>> {
        match self.data.get("MediaBox") {
            Some(PDFObject::Arrray(arrs)) => {
                let lx = arrs[0].as_f64()?;
                let ly = arrs[1].as_f64()?;
                let ux = arrs[2].as_f64()?;
                let uy = arrs[3].as_f64()?;
                Ok(Some(Rectangle::new(lx, ly, ux, uy)))
            }
            Some(obj) => Err(PDFError::InvalidContentSyntax(format!(
                "page mediabox not a rectanble,{:?}",
                obj
            ))),
            None => match self.parent {
                Some(ref p) => p.upgrade().unwrap().borrow().media_bbox(),
                None => Ok(None),
            },
        }
    }

    pub fn crop_bbox(&self) -> PDFResult<Option<Rectangle>> {
        match self.data.get("CropBox") {
            Some(PDFObject::Arrray(arrs)) => {
                let lx = arrs[0].as_f64()?;
                let ly = arrs[1].as_f64()?;
                let ux = arrs[2].as_f64()?;
                let uy = arrs[3].as_f64()?;
                Ok(Some(Rectangle::new(lx, ly, ux, uy)))
            }
            Some(obj) => Err(PDFError::InvalidContentSyntax(format!(
                "page mediabox not a rectanble,{:?}",
                obj
            ))),
            None => match self.parent {
                Some(ref p) => p.upgrade().unwrap().borrow().media_bbox(),
                None => Ok(None),
            },
        }
    }
}

#[derive(Default, Debug)]
pub struct PageTree {
    root: PageNodeRef,
    pages: HashMap<u32, PageNodeRef>,
}

impl PageTree {
    pub fn try_new<T: Seek + Read>(dict: PDFDictionary, doc: &Document<T>) -> PDFResult<Self> {
        if let Some(pagesref) = dict.get("Pages") {
            let pages: PDFDictionary = doc.read_indirect(pagesref)?.try_into()?;
            let root = create_pagetree(pages, doc, None)?;
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

    pub fn count(&self) -> PDFResult<u32> {
        let count = self.root.borrow().data().get("Count").unwrap().as_u32()?;
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

fn create_pagetree<T: Seek + Read>(
    root: PDFDictionary,
    doc: &Document<T>,
    parent: Option<Weak<RefCell<PageNode>>>,
) -> PDFResult<PageNodeRef> {
    let node = PageNode::new(root.clone(), parent);
    let noderef = Rc::new(RefCell::new(node));
    if let Some(PDFObject::Arrray(kids)) = root.get("Kids") {
        for kid in kids {
            let kid_data: PDFDictionary = doc.read_indirect(kid)?.try_into()?;
            let child = create_pagetree(kid_data, doc, Some(Rc::downgrade(&noderef)))?;
            noderef.borrow_mut().add_kid(child);
        }
    }
    Ok(noderef)
}
