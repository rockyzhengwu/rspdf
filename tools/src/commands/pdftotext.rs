use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;

use log::info;

use pdf::canvas::processor::Processor;
use pdf::device::text::TextDevice;
use pdf::document::Document;
use pdf::errors::PDFResult;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
}

pub fn command<T: Seek + Read>(
    doc: Document<T>,
    start: u32,
    end: u32,
    cfg: Config,
) -> PDFResult<()> {
    let device = Rc::new(RefCell::new(TextDevice::new()));
    let mut processor = Processor::new(&doc, device.clone());
    for p in start..end {
        info!("process page:{}", p);
        let page = doc.page(p).unwrap();
        processor.process_page_content(page, p).unwrap();
    }
    let outname = cfg.output.unwrap_or(PathBuf::from("pdftotxt.xml"));
    let mut file = File::create(outname).unwrap();
    file.write_all(device.borrow().result().as_bytes()).unwrap();
    Ok(())
}
