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
        // user cpace -> device
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

    fn show_text(&mut self, textinfo: &mut TextInfo) -> PDFResult<()> {
        // TODO wmode
        //
        let cids = textinfo.cids();
        let unicode = textinfo.get_unicode(cids.as_slice());
        let text_matrix = textinfo.text_matrix();

        // Convert user space to device space
        let ctm = textinfo.get_ctm().mutiply(&self.ctm);

        // font space to device space
        let scale = (f64::sqrt((ctm.v11 * ctm.v11 + ctm.v22 * ctm.v22) * 0.5)
            * textinfo.font_size()
            * text_matrix.v11) as u32;

        for cid in cids {
            let (ox, oy) = textinfo.out_pos(&cid, &ctm);
            if ox < 0.0 || oy < 0.0 {
                warn!("invalid position for {:?}, ", unicode);
            }
            //println!(
            //    "{},{},{},{},{}",
            //    cid,
            //    ox,
            //    oy,
            //    unicode,
            //    textinfo.font().name()
            //);
            match textinfo.font().get_glyph(&cid, &scale) {
                Some(glyph) => {
                    let bitmap = glyph.bitmap();
                    let dx = ox as u32;
                    let dy = oy as u32 - glyph.bitmap_top() as u32;
                    self.draw_char(dx, dy, &bitmap);
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
