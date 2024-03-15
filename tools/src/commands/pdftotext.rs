use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

use clap::Parser;

use log::info;

use pdf::device::text::TextDevice;
use pdf::device::Device;
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
    path: PathBuf,
) -> PDFResult<()> {
    let mut device = TextDevice::new();
    let filename = path.file_name().unwrap();
    for p in start..end {
        info!("process page:{}", p);
        let page = doc.get_page(&p).unwrap();
        device.start_page(p, page.bbox().unwrap());
        let objects = page.grapics_objects()?;
        for obj in objects {
            device.process(&obj).unwrap();
        }
        let outname = cfg.output.clone().unwrap_or(PathBuf::from(format!(
            "rspdf_{:?}_page_{}.txt",
            filename, p
        )));
        let mut file = File::create(outname).unwrap();
        file.write_all(device.result().as_bytes()).unwrap();
    }
    Ok(())
}
