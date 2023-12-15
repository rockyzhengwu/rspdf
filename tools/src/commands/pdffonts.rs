use std::collections::HashMap;
use std::fs::File;

use clap::Parser;

use pdf::document::Document;
use pdf::errors::PDFResult;
use pdf::object::{PDFDictionary, PDFObject};

#[derive(Debug, Parser)]
pub struct Config {}

#[derive(Debug)]
struct FontInfo {
    base_font: String,
    font_type: String,
    encoding: String,
}

pub fn command(doc: Document<File>, start: u32, end: u32, _cfg: Config) -> PDFResult<()> {
    let mut allfonts: HashMap<String, FontInfo> = HashMap::new();
    for p in start..end {
        let page = doc.page(p).unwrap();
        let resource = page.borrow().resources();
        if resource.is_indirect() {
            let resobj = doc.read_indirect(&resource).unwrap();
            let fonts = match resobj.get_value("Font").unwrap() {
                PDFObject::Dictionary(d) => PDFObject::Dictionary(d.to_owned()),
                PDFObject::Indirect(r) => doc
                    .read_indirect(&PDFObject::Indirect(r.to_owned()))
                    .unwrap(),
                _ => panic!("fonts type error"),
            };
            let fonts_dict: PDFDictionary = fonts.try_into().unwrap();
            for (key, obj) in fonts_dict.iter() {
                let font_obj = doc.read_indirect(obj).unwrap();
                let encoding = font_obj
                    .get_value("Encoding")
                    .unwrap_or(&PDFObject::Null)
                    .as_string()
                    .unwrap();
                let font_type = font_obj.get_value("Subtype").unwrap().as_string().unwrap();
                let base_font = font_obj.get_value("BaseFont").unwrap().as_string().unwrap();
                let name = key.to_string();
                let finfo = FontInfo {
                    base_font,
                    font_type,
                    encoding,
                };
                allfonts.entry(name).or_insert(finfo);
            }
        }
    }
    for (name, info) in allfonts {
        println!("{:?},{:?}", name, info);
    }
    Ok(())
}
