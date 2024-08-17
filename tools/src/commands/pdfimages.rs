use std::io::{Read, Seek};
use std::path::PathBuf;

use clap::Parser;
use image::{Rgb, RgbImage};

use log::info;
use pdf::color::RGBValue;
use pdf::document::Document;
use pdf::errors::PDFResult;
use pdf::page::graphics_object::GraphicsObject;

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
    let filename = path.file_name().unwrap();
    println!("{:?},{:?}", start, end);
    for p in start..end {
        info!("Process page: {}", p);
        let page = doc.get_page(&p).unwrap();
        let objects = page.grapics_objects()?;
        let mut n = 0;

        for obj in objects {
            if let GraphicsObject::Image(image) = obj {
                // TODO ignore mask now
                if image.is_mask() {
                    continue;
                }
                let outpath = format!("rspdf_render_{:?}_{}_{}.png", filename, p, n);
                let w = image.width()?;
                let h = image.height()?;
                let pixmap = image.rgb_image()?;

                let mut im = RgbImage::new(w as u32, h as u32);
                for i in 0..(h as u32) {
                    for j in 0..(w as u32) {
                        let index = i * (w as u32) + j;
                        let pixel: &RGBValue = pixmap.get(index as usize).unwrap();
                        im.put_pixel(j, i, Rgb([pixel.r(), pixel.g(), pixel.b()]));
                    }
                }
                im.save(outpath).unwrap();

                n += 1;
            }
        }
    }
    Ok(())
}
