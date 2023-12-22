use freetype::Bitmap;
use image::{Rgb, RgbImage};

use log::warn;

use crate::canvas::path_info::PathInfo;
use crate::canvas::text_info::TextInfo;
use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;

pub struct ImageDevice {
    x_res: f64,
    y_res: f64,
    image: RgbImage,
}

impl ImageDevice {
    pub fn new(x_res: f64, y_res: f64) -> Self {
        let image = RgbImage::new(1, 1);
        ImageDevice {
            x_res,
            y_res,
            image,
        }
    }
}
impl ImageDevice {
    fn draw_char(&mut self, x: u32, y: u32, bitmap: &Bitmap) {
        let width = bitmap.width() as u32;
        let height = bitmap.rows() as u32;
        let buffer = bitmap.buffer();
        for i in 0..height {
            for j in 0..width {
                let pixel = buffer[(i * width + j) as usize];
                if pixel == 0 {
                    continue;
                }
                let rgb = Rgb([0, 0, 0]);
                self.image
                    .put_pixel(x + j, self.image.height() - y + i, rgb);
            }
        }
    }
}

impl Device for ImageDevice {
    fn begain_page(&mut self, _page_num: u32, media: &Rectangle, _crop: &Rectangle) {
        let width = self.x_res / 72.0 * media.width();
        let height = self.y_res / 72.0 * media.height();

        self.image = RgbImage::from_fn(width as u32, height as u32, |_, _| {
            image::Rgb([255, 255, 255])
        });
    }

    fn end_page(&mut self, page_num: u32) {
        self.image.save(format!("page-{}.png", page_num)).unwrap()
    }

    fn show_text(&mut self, mut textinfo: TextInfo) -> PDFResult<()> {
        // bitmap, x, y for every character
        // TODO Encoding PDFString-> Character Encoding, multi bytes may be one character
        // TODO color
        let unicode = textinfo.get_unicode();

        let (x, y) = textinfo.position();
        // some text position is negative , just ignore
        if x < 0.0 || y < 0.0 {
            warn!("content out of device bound:{},{},{}", x, y, unicode);
            return Ok(());
        }
        // println!("{:?},{:?},{:?}", x, y, unicode);

        let bbox = textinfo.bbox();
        let sx = self.image.width() as f64 / bbox.width();
        let sy = self.image.width() as f64 / bbox.height();
        let mut x = x * sx;
        let y = y * sx;
        let scale = f64::sqrt((sx * sx + sy * sy) / 2.0);
        let cids = textinfo.cids();
        for cid in cids {
            let w = textinfo.get_character_width(cid);
            let bitmap = textinfo.get_glyph(cid, scale);
            if bitmap.is_none() {
                warn!("bitmap is NOne");
                continue;
            }
            let bitmap = bitmap.unwrap();
            self.draw_char(x as u32, (y + bitmap.rows() as f64) as u32, &bitmap);
            x += w * sx;
        }

        Ok(())
    }

    fn paint_path(&mut self, _pathinfo: PathInfo) -> PDFResult<()> {
        Ok(())
    }
}
