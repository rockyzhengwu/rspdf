use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use pdf::device::trace::Trace;
use pdf::document::Document;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
}

pub fn command(doc: &Document, config: Config, start: u32, end: u32) {
    let mut device = Trace::new();
    for p in start..end {
        if let Some(page) = doc.get_page(&p) {
            page.display(p, &mut device).unwrap();
        } else {
            panic!("Document page {} dosen't exist", p);
        }
    }
    let mut out = std::fs::File::create("trace.xml").unwrap();
    out.write_all(device.content().as_bytes()).unwrap();
}
