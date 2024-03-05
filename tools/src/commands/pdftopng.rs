use std::io::{Read, Seek};

use clap::Parser;

use log::info;
use pdf::device::image_device::ImageDevice;
use pdf::device::Device;
use pdf::document::Document;
use pdf::errors::PDFResult;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long, default_value_t = 300.0)]
    resulotion: f64,
}

pub fn command<T: Seek + Read>(
    doc: Document<T>,
    start: u32,
    end: u32,
    cfg: Config,
) -> PDFResult<()> {
    let mut device = ImageDevice::new(cfg.resulotion, cfg.resulotion);
    for p in start..end {
        info!("Process page: {}", p);
        let page = doc.get_page(&p).unwrap();
        let objects = page.grapics_objects()?;
        for obj in objects {
            device.process(&obj)?;
        }
    }
    Ok(())
}
