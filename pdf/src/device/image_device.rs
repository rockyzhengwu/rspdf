use freetype::Bitmap;
use image::{Rgba, RgbaImage};

use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::matrix::Matrix;

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
    fn process(&mut self, obj: &crate::page::graphics_object::GraphicsObject) -> PDFResult<()> {
        println!("{:?}", obj);
        Ok(())
    }
}
