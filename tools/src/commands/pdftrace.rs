use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

use clap::Parser;

use log::info;

use pdf::device::trace::TraceDevice;
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
    let mut device = TraceDevice::new(path.display().to_string().as_str());
    for p in start..end {
        info!("process page:{}", p);
        let page = doc.get_page(&p).unwrap();
        let object_iterator = page.grapics_objects()?;
        device.start_page(p, page.bbox()?);
        for obj in object_iterator {
            device.process(&obj)?;
        }
    }
    let outname = format!("rspdf_trace_{:?}.xml", path.file_name().unwrap());
    let outname = cfg.output.unwrap_or(PathBuf::from(outname.as_str()));
    let mut file = File::create(outname).unwrap();
    file.write_all(device.result().as_bytes()).unwrap();
    Ok(())
}
