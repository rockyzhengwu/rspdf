use std::io::{Read, Seek};
use std::path::PathBuf;

use clap::Parser;

use log::info;
use pdf::device::cairo::CairoRender;
use pdf::device::Device;
use pdf::document::Document;
use pdf::errors::PDFResult;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long, default_value_t = 72.0)]
    resulotion: f64,
}

pub fn command<T: Seek + Read>(
    doc: Document<T>,
    start: u32,
    end: u32,
    cfg: Config,
    path: PathBuf,
) -> PDFResult<()> {
    let mut device = CairoRender::new(cfg.resulotion);
    let filename = path.file_name().unwrap();
    for p in start..end {
        info!("Process page: {}", p);
        let page = doc.get_page(&p).unwrap();
        let objects = page.grapics_objects()?;
        device.start_page(p, page.bbox()?);
        for obj in objects {
            device.process(&obj)?;
        }
        let outpath = format!("rspdf_render_{:?}_{}.png", filename, p);
        device.save_image(PathBuf::from(outpath));
    }
    Ok(())
}
