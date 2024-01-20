use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;

use log::info;

use pdf::device::trace::TraceDevice;
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
    let device = Rc::new(RefCell::new(TraceDevice::new("datastructure.pdf")));
    for p in start..end {
        info!("process page:{}", p);
        let page = doc.get_page(&p).unwrap();
        page.display(device.clone()).unwrap();
    }
    let outname = cfg.output.unwrap_or(PathBuf::from("pdftrace.xml"));
    let mut file = File::create(outname).unwrap();
    file.write_all(device.borrow_mut().result().as_bytes())
        .unwrap();
    Ok(())
}
