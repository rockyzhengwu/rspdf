use freetype::Bitmap;
use image::{Rgba, RgbaImage};

use log::warn;

use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::matrix::Matrix;
use crate::geom::path::Path;
use crate::geom::rectangle::Rectangle;
use crate::page::text::Text;

pub struct ImageDevice {
    x_res: f64,
    y_res: f64,
    image: RgbaImage,
    ctm: Matrix,
}

impl ImageDevice {
    pub fn new(x_res: f64, y_res: f64) -> Self {
        let w = (x_res * 72.0) as u32;
        let h = (x_res * 72.0) as u32;
        let image = RgbaImage::new(w, h);
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
    fn begain_page(&mut self, _page_num: &u32, media: Option<Rectangle>, crop: Option<Rectangle>) {
        println!("begain_page {:?},{:?}", media, crop);
        let bbox = match (media, crop) {
            (Some(m), Some(c)) => m,
            (None, Some(c)) => c,
            (Some(m), None) => m,
            (None, None) => Rectangle::new(0.0, 0.0, 72.0, 72.0),
        };
        let sx = self.x_res / 72.0;
        let sy = self.y_res / 72.0;
        // user cpace -> device
        let ctm = Matrix::new(
            sx,
            0.0,
            0.0,
            -1.0 * sy,
            -1.0 * sx * bbox.lx(),
            sy * bbox.uy(),
        );
        self.ctm = ctm;
        let width = (sx * (bbox.width() + 0.5)) as u32;
        let height = (sy * (bbox.height() + 0.5)) as u32;

        self.image = RgbaImage::from_fn(width, height, |_, _| image::Rgba([255, 255, 255, 255]));
    }

    fn end_page(&mut self, page_num: &u32) {
        self.image.save(format!("page-{}.png", page_num)).unwrap()
    }

    fn start_text(&mut self) {}

    fn show_text(&mut self, textobj: &Text) -> PDFResult<()> {
        let ctm = textobj.ctm().mutiply(&self.ctm);
        let font = textobj.font();
        let font_size = textobj.font_size();
        for item in textobj.items() {
            let scale = (ctm.v11 * font_size * item.tm().v11) as u32;
            let code = item.code();
            let bbox = item.bbox();
            let lx = bbox.lx();
            let ly = bbox.ly();

            let x = lx * ctm.v11 + ly * ctm.v21 + ctm.v31;
            let y = lx * ctm.v12 + ly * ctm.v22 + ctm.v32;

            if let Some(glyph) = font.get_glyph(code, &scale) {
                let bitmap = glyph.bitmap();
                let y = y - glyph.bitmap_top() as f64;
                let x = x + glyph.bitmap_left() as f64;
                self.draw_char(x as u32, y as u32, &bitmap)
            } else {
                warn!("{:?},didn't found glyph", item)
            }
        }
        Ok(())
    }

    fn end_text(&mut self) {}

    fn paint_path(&mut self, path: &Path) -> PDFResult<()> {
        let current = path.current_point();
        for sub in path.sub_paths() {
            for seg in sub.segments() {
                match seg {
                    Bezier => {
                        println!("bezier:{:?}", seg.points());
                    }
                    Line => {}
                }
            }
        }
        Ok(())
    }
}
