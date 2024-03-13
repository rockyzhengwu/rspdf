use freetype::Bitmap;
use image::{Rgba, RgbaImage};
use log::warn;

use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::matrix::Matrix;
use crate::geom::rectangle::Rectangle;
use crate::page::graphics_object::GraphicsObject;

#[derive(Default)]
pub struct ImageDevice {
    image: RgbaImage,
    ctm: Matrix,
    scale_x: f64,
    scale_y: f64,
}

impl ImageDevice {
    pub fn new(res: f64) -> Self {
        let scale_x = res / 72.0;
        let scale_y = scale_x;
        ImageDevice {
            scale_x,
            scale_y,
            ctm: Matrix::default(),
            image: RgbaImage::default(),
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
    pub fn finish_page(&self) {
        println!("finish");
        self.image.save(format!("page-{}.png", 3)).unwrap()
    }
}

impl Device for ImageDevice {
    fn start_page(&mut self, bbox: Rectangle) {
        let pw = bbox.width();
        let ph = bbox.height();
        let w = ((pw + 1.0) * self.scale_x) as u32;
        let h = ((ph + 1.0) * self.scale_x) as u32;
        self.ctm = Matrix::new(
            self.scale_x,
            0.0,
            0.0,
            -1.0 * self.scale_y,
            -1.0 * self.scale_x * bbox.lx(),
            self.scale_y * bbox.uy(),
        );
        self.image = RgbaImage::from_fn(w, h, |_, _| image::Rgba([255, 255, 255, 255]));
    }

    fn process(&mut self, obj: &GraphicsObject) -> PDFResult<()> {
        match obj {
            GraphicsObject::Path(_) => {}
            GraphicsObject::Text(text) => {
                let font = text.font();
                let font_size = text.font_size();
                let mut text_matrix = text.text_matrix().to_owned();
                let char_spacing = text.char_spacing();
                let horz_scale = text.text_horz_scale();
                let word_spacing = text.word_space();
                let text_rise = text.text_rise();
                let ctm = text.ctm().mutiply(&self.ctm);
                for con in text.text_items() {
                    let tj = (-con.adjust() * 0.001) * font_size * horz_scale;
                    let mrm = Matrix::new_translation_matrix(tj, 0.0);
                    text_matrix = mrm.mutiply(&text_matrix);

                    let chars = font.decode_chars(con.bytes());
                    for char in chars.iter() {
                        let mut displacement = font.get_char_width(char) * 0.001 + char_spacing;
                        if char.is_space() {
                            displacement += word_spacing;
                        }
                        let trm = Matrix::new(
                            font_size * horz_scale,
                            0.0,
                            0.0,
                            font_size,
                            0.0,
                            text_rise,
                        );
                        let user_matrix = trm.mutiply(&text_matrix).mutiply(&ctm);
                        let x = user_matrix.v31 as u32;
                        let y = user_matrix.v32 as u32;
                        let scale_x = user_matrix.v11;
                        let scale_y = user_matrix.v22;
                        let scale = ((scale_y * scale_y + scale_x * scale_x) * 0.5).sqrt() as u32;
                        if let Some(gid) = font.glyph_index_from_charcode(char) {
                            if let Some(glyph) = font.get_glyph(gid, scale) {
                                let bitmap = glyph.bitmap();
                                let y = y - glyph.bitmap_top() as u32;
                                self.draw_char(x, y, &bitmap);
                            } else {
                                warn!("glyph is None");
                            }
                        }
                        let mrm = Matrix::new_translation_matrix(displacement, 0.0);
                        text_matrix = mrm.mutiply(&text_matrix);
                        // move
                    }
                }
            }
            GraphicsObject::Image(_) => {}
        }
        Ok(())
    }
}
