use std::path::PathBuf;

use crate::device::image_device::ImageDevice;
use clap::Parser;
use pdf::document::Document;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
}

pub fn command(doc: &Document, config: Config, start: u32, end: u32) {
    let mut device = ImageDevice::new();
    for p in start..end {
        if let Some(page) = doc.get_page(&p) {
            page.display(p, &mut device).unwrap();
        } else {
            panic!("Document page {} dosen't exist", p);
        }
    }

    println!("pdftrace {:?}", config);
}
