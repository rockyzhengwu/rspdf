use freetype::Bitmap;
use image::{Rgb, RgbImage};

use crate::canvas::matrix::Matrix;

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
    ctm: Matrix,
}

impl ImageDevice {
    pub fn new(x_res: f64, y_res: f64) -> Self {
        let image = RgbImage::new(1, 1);
        ImageDevice {
            x_res,
            y_res,
            image,
            ctm: Matrix::default(),
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
                self.image.put_pixel(x + j, y + i, rgb);
            }
        }
    }
}

impl Device for ImageDevice {
    fn begain_page(&mut self, _page_num: u32, media: &Rectangle, _crop: &Rectangle) {
        let sx = self.x_res / 72.0;
        let sy = self.y_res / 72.0;
        let ctm = Matrix::new(
            sx,
            0.0,
            0.0,
            -1.0 * sy,
            -1.0 * sx * media.x(),
            sy * media.height(),
        );
        self.ctm = ctm;
        let width = (sx * (media.width() - media.x() + 0.5)) as u32;
        let height = (sy * (media.height() - media.y() + 0.5)) as u32;

        self.image = RgbImage::from_fn(width, height, |_, _| image::Rgb([255, 255, 255]));
    }

    fn end_page(&mut self, page_num: u32) {
        self.image.save(format!("page-{}.png", page_num)).unwrap()
    }

    fn show_text(&mut self, mut textinfo: TextInfo) -> PDFResult<()> {
        // TODO implement render
        let unicode = textinfo.get_unicode();

        let (x, y) = textinfo.position();
        // some text position is negative , just ignore
        if x < 0.0 || y < 0.0 {
            warn!("content out of device bound:{},{},{}", x, y, unicode);
            return Ok(());
        }

        // println!("draw: {:?},{:?},{:?}", x, y, unicode,);

        let sx = self.x_res / 72.0;
        let sy = self.y_res / 72.0;
        // TODO calc font size
        let scale = f64::sqrt((sx * sx + sy * sy) / 4.0);
        let cids = textinfo.cids();
        let ctm = textinfo.get_ctm().mutiply(&self.ctm);
        for cid in cids {
            let (ox, oy) = textinfo.out_pos(cid, &ctm);
            let bitmap = textinfo.get_glyph(cid, scale);
            if bitmap.is_none() {
                panic!("bitmap is NOne");
            }
            let bitmap = bitmap.unwrap();
            self.draw_char(ox, oy - bitmap.rows() as u32, &bitmap);
        }
        Ok(())
    }

    fn paint_path(&mut self, _pathinfo: PathInfo) -> PDFResult<()> {
        Ok(())
    }
}
