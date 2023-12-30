use freetype::Bitmap;
use image::{Rgba, RgbaImage};

use log::warn;

use crate::canvas::matrix::Matrix;
use crate::canvas::path_info::PathInfo;
use crate::canvas::text_info::TextInfo;
use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;

pub struct ImageDevice {
    x_res: f64,
    y_res: f64,
    image: RgbaImage,
    ctm: Matrix,
}

impl ImageDevice {
    pub fn new(x_res: f64, y_res: f64) -> Self {
        let image = RgbaImage::new(1, 1);
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
                let rgb = Rgba([0, 0, 0, pixel]);
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
            -1.0 * sx * media.lx(),
            sy * media.uy(),
        );
        self.ctm = ctm;
        let width = (sx * (media.width() + 0.5)) as u32;
        let height = (sy * (media.height() + 0.5)) as u32;

        self.image = RgbaImage::from_fn(width, height, |_, _| image::Rgba([255, 255, 255, 255]));
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
        let scale = f64::sqrt((sx * sx + sy * sy) * 0.5);
        let cids = textinfo.cids();
        let ctm = textinfo.get_ctm().mutiply(&self.ctm);
        for cid in cids {
            let (tx, ty) = textinfo.out_pos(cid, &ctm);
            match textinfo.get_glyph(cid, scale) {
                Some(glyph) => {
                    let bitmap = glyph.bitmap();
                    let ux = (tx as i32 + glyph.bitmap_left()) as u32;
                    let uy = ty - glyph.bitmap_top() as u32;
                    self.draw_char(ux, uy, &bitmap);
                }
                None => {
                    panic!("bitmap is NOne");
                }
            }
        }
        Ok(())
    }

    fn paint_path(&mut self, _pathinfo: PathInfo) -> PDFResult<()> {
        Ok(())
    }
}
