use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::PathBuf;

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
    tounicode: bool,
}

fn parse_basefont(name: String) -> String {
    if name.len() > 7 && name.chars().nth(6) == Some('+') {
        let (_, last) = name.split_at(7);
        last.to_string()
    } else {
        name
    }
}

pub fn command<T: Seek + Read>(
    doc: Document<T>,
    start: u32,
    end: u32,
    _cfg: Config,
    _path: PathBuf,
) -> PDFResult<()> {
    let mut allfonts: HashMap<String, FontInfo> = HashMap::new();
    for p in start..end {
        info!("process page :{:?}", p);
        let page = doc.get_page(&p).unwrap();
        let resources = page.resources()?;
        if let Some(fontref) = resources.get_value("Font") {
            let fonts_dict: PDFDictionary = match fontref {
                PDFObject::Indirect(_) => doc.read_indirect(fontref).unwrap().try_into().unwrap(),
                PDFObject::Dictionary(d) => d.to_owned(),
                _ => {
                    panic!("Font data type error")
                }
            };
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
                let base_font = parse_basefont(base_font);
                let tounicode = font_obj.get_value("ToUnicode");
                let name = key.to_string();
                let finfo = FontInfo {
                    base_font,
                    font_type,
                    encoding,
                    tounicode: tounicode.is_some(),
                };
                if allfonts.contains_key(&name) {
                    continue;
                }
                allfonts.entry(name).or_insert(finfo);
            }
        }
    }
    for (name, info) in allfonts {
        println!("{:?},{:?}", name, info);
    }
    Ok(())
}
