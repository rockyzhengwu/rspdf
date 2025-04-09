use clap::Parser;
use pdf::document::Document;

#[derive(Debug, Parser)]
pub struct Config {}

pub fn command(doc: &Document, config: Config, start: u32, end: u32) {

    //let page = doc.get_page(&2).unwrap();
    //let resources = page.resources();
    //if let Some(fonts) = resources.fonts() {
    //    for (key, val) in fonts.entries() {
    //        let font_obj = doc.read_object(val).unwrap();
    //        let sub_type = font_obj.get_from_dict("Subtype").unwrap();
    //        let desc = font_obj.get_from_dict("FontDescriptor").unwrap();
    //        let desc_obj = doc.read_object(desc);
    //        println!("{:?}, {:?}", key, desc_obj);
    //    }
    //}
}
