use crate::object::PdfObject;

#[derive(Debug, PartialEq, Clone)]
pub struct PdfArray {
    elements: Vec<PdfObject>,
}
impl PdfArray {
    pub fn new(elements: Vec<PdfObject>) -> Self {
        Self { elements }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn get(&self, index: usize) -> Option<&PdfObject> {
        self.elements.get(index)
    }

    pub fn iter(&self) -> PdfArrayIterator {
        PdfArrayIterator {
            array: self,
            index: 0,
        }
    }
}

pub struct PdfArrayIterator<'a> {
    array: &'a PdfArray,
    index: usize,
}

impl<'a> Iterator for PdfArrayIterator<'a> {
    type Item = &'a PdfObject;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.len() {
            return None;
        }
        let obj = self.array.get(self.index);
        self.index += 1;
        obj
    }
}
