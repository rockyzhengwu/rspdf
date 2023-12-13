use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

use pdf::canvas::processor::Processor;
use pdf::device::text::TextDevice;
use pdf::document::Document;
use pdf::errors::PDFResult;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long)]
    output: PathBuf,
}

pub fn command(doc: Document<File>, start: u32, end: u32, cfg: Config) -> PDFResult<()> {
    let mut device = TextDevice::new();
    let mut processor = Processor::new(&doc, &mut device);
    for p in start..end {
        let page = doc.page(p).unwrap();
        processor.process_page_content(page).unwrap();
    }
    let mut file = File::create(cfg.output).unwrap();
    file.write_all(device.result().as_bytes()).unwrap();
    Ok(())
}
