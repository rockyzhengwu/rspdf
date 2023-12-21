use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

use clap::Parser;

use log::info;
use pdf::canvas::processor::Processor;
use pdf::device::image_device::ImageDevice;
use pdf::document::Document;
use pdf::errors::PDFResult;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long, default_value_t = 300.0)]
    resulotion: f64,
}

pub fn command(doc: Document<File>, start: u32, end: u32, cfg: Config) -> PDFResult<()> {
    let device = Rc::new(RefCell::new(ImageDevice::new(
        cfg.resulotion,
        cfg.resulotion,
    )));
    let mut processor = Processor::new(&doc, device);
    for p in start..end {
        info!("Process page: {}", p);
        let page = doc.page(p).unwrap();
        processor.process_page_content(page, p).unwrap();
    }
    Ok(())
}
