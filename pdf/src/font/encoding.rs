use crate::object::PDFObject;

// WiinAnsiEncoding
// Identity-H
// qlq

#[allow(unused)]
pub struct FontEncoding {
    encoding: Vec<u8>,
}


#[allow(unused)]
impl FontEncoding {
    pub fn new(obj: PDFObject) -> Self {
        let encoding = Vec::new();
        FontEncoding { encoding }
    }
}
