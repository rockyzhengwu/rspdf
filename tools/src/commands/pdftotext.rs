use std::path::PathBuf;

use crate::device::text_device::TextDevice;
use clap::Parser;
use pdf::document::Document;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
}

pub fn command(doc: &Document, _config: Config, start: u32, end: u32) {
    let mut device = TextDevice::default();
    for p in start..end {
        if let Some(page) = doc.get_page(&p) {
            println!("tart_process_page: {:?}", p);
            page.display(p, &mut device).unwrap();
            let page_content = device.page_content();
            println!("page_content:{:?}", page_content);
        } else {
            panic!("Document page {} dosen't exist", p);
        }
    }
}
