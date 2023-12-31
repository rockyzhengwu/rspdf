use std::collections::HashMap;
use std::io::{Read, Seek};

use clap::Parser;
use log::info;

use pdf::document::Document;
use pdf::errors::PDFResult;
use pdf::object::{PDFDictionary, PDFObject};

#[derive(Debug, Parser)]
pub struct Config {}

#[allow(dead_code)]
#[derive(Debug)]
struct FontInfo {
    base_font: String,
    font_type: String,
    encoding: String,
}

pub fn command<T: Seek + Read>(
    doc: Document<T>,
    start: u32,
    end: u32,
    _cfg: Config,
) -> PDFResult<()> {
    let mut allfonts: HashMap<String, FontInfo> = HashMap::new();
    for p in start..end {
        info!("processing page:{}", p);
        let page = doc.page(p).unwrap();
        let resource = page.borrow().resources();
        let resobj = doc.read_indirect(&resource).unwrap();
        let fontref = resobj.get_value("Font");
        if fontref.is_none() {
            continue;
        }
        let fonts = doc.read_indirect(fontref.unwrap()).unwrap();
        let fonts_dict: PDFDictionary = fonts.try_into().unwrap();
        for (key, obj) in fonts_dict.iter() {
            let font_obj = doc.read_indirect(obj).unwrap();
            let enc_obj = font_obj.get_value("Encoding");
            let encoding = match enc_obj {
                Some(&PDFObject::Indirect(_)) => "Embedding".to_string(),
                Some(PDFObject::Name(n)) => n.to_string(),
                _ => "".to_string(),
            };

            let font_type = font_obj.get_value("Subtype").unwrap().as_string().unwrap();
            let base_font = font_obj.get_value("BaseFont").unwrap().as_string().unwrap();
            let _tounicode = font_obj.get_value("ToUnicode");
            let name = key.to_string();
            let finfo = FontInfo {
                base_font,
                font_type,
                encoding,
            };
            println!("{:?},{:?}", name, obj);
            allfonts.entry(name).or_insert(finfo);
        }
    }
    for (name, info) in allfonts {
        println!("{:?},{:?}", name, info);
    }
    Ok(())
}
