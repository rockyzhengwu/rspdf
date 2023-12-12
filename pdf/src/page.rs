use std::cell::RefCell;
use std::io::{Read, Seek};
use std::rc::{Rc, Weak};

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::geom::rectangle::Rectangle;
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub enum PageType {
    Intermediate,
    Leaf,
}

#[derive(Debug)]
pub struct Page {
    parent: Option<Weak<RefCell<Page>>>,
    children: Vec<PageRef>,
    dict: PDFObject,
    count: u32,
    t: PageType,
}

pub type PageRef = Rc<RefCell<Page>>;

impl Page {
    pub fn new(dict: PDFObject, parent: Option<Weak<RefCell<Page>>>) -> Self {
        let count = dict.get_value_as_i64("Count").map_or(Ok(1), |v| v).unwrap();
        // TODO fix unwrap
        let t: PageType = match dict.get_value_as_string("Type").unwrap().unwrap().as_str() {
            "Pages" => PageType::Intermediate,
            "Page" => PageType::Leaf,
            _ => PageType::Leaf,
        };
        Page {
            dict,
            parent,
            children: Vec::new(),
            count: count as u32,
            t,
        }
    }

    pub fn add_child(&mut self, child: PageRef) {
        self.children.push(child)
    }

    pub fn info(&self) -> &PDFObject {
        &self.dict
    }

    pub fn page_type(&self) -> PageType {
        self.t.to_owned()
    }

    pub fn get_value(&self, key: &str) -> Option<&PDFObject> {
        self.dict.get_value(key)
    }

    pub fn media_box(&self) -> Option<Rectangle> {
        // TODO unwrap
        match self.dict.get_value("MediaBox") {
            Some(PDFObject::Arrray(ref arr)) => {
                let position: Vec<f64> = arr.iter().map(|v| v.clone().as_f64().unwrap()).collect();
                Some(Rectangle::new(
                    position[0],
                    position[1],
                    position[2],
                    position[3],
                ))
            }
            _ => None,
        }
    }
    pub fn crop_box(&self) -> Option<Rectangle> {
        match self.dict.get_value("CropBox") {
            Some(PDFObject::Arrray(ref arr)) => {
                let position: Vec<f64> = arr.iter().map(|v| v.clone().as_f64().unwrap()).collect();
                Some(Rectangle::new(
                    position[0],
                    position[1],
                    position[2],
                    position[3],
                ))
            }
            _ => None,
        }
    }

    pub fn contents(&self) -> PDFObject {
        // TODO contents is array or not
        self.dict
            .get_value("Contents")
            .map_or_else(|| PDFObject::Arrray(Vec::new()), |v| v.to_owned())
    }

    pub fn resources(&self) -> PDFObject {
        // TODO handle this panic
        match self.dict.get_value("Resources") {
            Some(v) => v.to_owned(),
            None => match self.parent {
                Some(ref p) => p.upgrade().unwrap().borrow().resources(),
                None => panic!("no resource in page"),
            },
        }
    }
}

pub struct PageTree {
    root: PageRef,
    count: i64,
}

impl PageTree {
    pub fn try_new<T: Seek + Read>(doc: &Document<T>) -> PDFResult<Option<Self>> {
        let catalog = doc.catalog();
        println!("{:?}", catalog);
        match catalog.get_value("Pages") {
            Some(obj) => {
                let root_page = doc.read_indirect(obj)?;
                let count = root_page.get_value_as_i64("Count").unwrap().unwrap();
                let root = build_page_tree(root_page.to_owned(), doc, None)?;
                Ok(Some(PageTree { root, count }))
            }
            None => Ok(None),
        }
    }

    pub fn get_page(&self, number: u32) -> Option<PageRef> {
        find_page(self.root.clone(), number)
    }

    pub fn page_count(&self) -> i64 {
        self.count
    }
}

pub fn build_page_tree<T: Seek + Read>(
    root: PDFObject,
    doc: &Document<T>,
    parent: Option<Weak<RefCell<Page>>>,
) -> PDFResult<PageRef> {
    let root_page = Rc::new(RefCell::new(Page::new(root.clone(), parent)));
    let root_weak = Rc::downgrade(&root_page);
    // TODO fix this
    match root
        .get_value_as_string("Type")
        .unwrap_or(Ok("".to_string()))?
        .as_str()
    {
        "Pages" => {
            // Kids must in pages
            let kids = root.get_value("Kids").unwrap().as_array()?;
            for kid in kids.iter() {
                let kid_obj = doc.read_indirect(kid)?;
                let child = build_page_tree(kid_obj, doc, Some(root_weak.clone()))?;
                root_page.borrow_mut().add_child(child);
            }
            Ok(root_page)
        }
        "Page" => Ok(root_page),
        _ => Err(PDFError::InvalidSyntax(
            "Page Root Type value must be Page or Pages".to_string(),
        )),
    }
}

pub fn find_page(node: PageRef, number: u32) -> Option<PageRef> {
    match node.borrow().page_type() {
        PageType::Intermediate => {
            let mut n = 0;
            for kid in node.borrow().children.iter() {
                n += kid.borrow().count;
                if n >= number {
                    return find_page(kid.clone(), kid.borrow().count - (n - number));
                }
            }
            // TODO Fix this
            None
        }
        PageType::Leaf => Some(node.clone()),
    }
}
