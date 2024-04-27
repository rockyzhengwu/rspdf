use crate::errors::PDFResult;
use crate::filter::Filter;
use crate::object::PDFObject;

#[derive(Default)]
pub struct CCITTFaxDecode {}

impl Filter for CCITTFaxDecode {
    fn decode(&self, buf: &[u8], param: Option<&PDFObject>) -> PDFResult<Vec<u8>> {
        
        println!("{:?}", param);
        unimplemented!()
    }
}
