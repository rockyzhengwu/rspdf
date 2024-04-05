use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cairo::{Context, FontFace, Format, Glyph, ImageSurface};
use image::{Rgb, RgbImage};

use crate::color::ColorRGBValue;
use crate::device::Device;
use crate::geom::matrix::Matrix;
use crate::geom::point::Point;
use crate::geom::subpath::Segment;
use crate::page::graphics_object::GraphicsObject;

pub struct CairoRender {
    surface: ImageSurface,
    ctm: Matrix,
    context: Context,
    scale: f64,
    current_page: u32,
}
impl CairoRender {
    pub fn new(scale: f64) -> Self {
        let surface = ImageSurface::create(Format::ARgb32, 600, 600).unwrap();
        let context = Context::new(&surface).unwrap();
        let scale = scale / 72.0;

        CairoRender {
            scale,
            current_page: 0_u32,
            surface,
            context,
            ctm: Matrix::default(),
        }
    }

    pub fn save_image(&self, path: PathBuf) {
        let mut file = File::create(path).unwrap();
        self.surface.write_to_png(&mut file).unwrap();
    }
}

fn transform_point(point: &Point, ctm: &Matrix) -> Point {
    let x = point.x() * ctm.v11 + point.y() * ctm.v21 + ctm.v31;
    let y = point.x() * ctm.v21 + point.y() * ctm.v22 + ctm.v32;
    Point::new(x, y)
}

impl Device for CairoRender {
    fn start_page(&mut self, num: u32, bbox: crate::geom::rectangle::Rectangle) {
        self.current_page = num;
        let pw = bbox.width();
        let ph = bbox.height();
        let w = ((pw + 1.0) * self.scale) as i32;
        let h = ((ph + 1.0) * self.scale) as i32;
        self.ctm = Matrix::new(
            self.scale,
            0.0,
            0.0,
            -1.0 * self.scale,
            -1.0 * self.scale * bbox.lx(),
            self.scale * bbox.uy(),
        );
        let surface = ImageSurface::create(Format::ARgb32, w, h).unwrap();
        let context = Context::new(&surface).unwrap();
        context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        context.paint().unwrap();
        self.surface = surface;
        self.context = context;
    }

    fn process(
        &mut self,
        obj: &crate::page::graphics_object::GraphicsObject,
    ) -> crate::errors::PDFResult<()> {
        match obj {
            GraphicsObject::Path(path) => {
                let line_width = path.line_width().to_owned();
                self.context.set_line_width(line_width * self.scale);
                self.context.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                // TODO set page state
                let ctm = path.ctm().mutiply(&self.ctm);
                for sub in path.path().sub_paths() {
                    for seg in sub.segments() {
                        match seg {
                            Segment::Rect(_) => {
                                //
                            }
                            Segment::Line(l) => {
                                let start = transform_point(l.start(), &ctm);
                                let end = transform_point(l.end(), &ctm);
                                self.context.move_to(start.x(), start.y());
                                self.context.line_to(end.x(), end.y());
                            }
                            Segment::Curve(c) => {}
                        }
                    }
                }
                self.context.stroke().unwrap();
            }

            GraphicsObject::Text(text) => {
                let font = text.font();
                let font_size = text.font_size();
                let mut text_matrix = text.text_matrix().to_owned();
                let char_spacing = text.char_spacing();
                let horz_scale = text.text_horz_scale();
                let word_spacing = text.word_space();
                let text_rise = text.text_rise();
                let ctm = text.ctm().mutiply(&self.ctm);
                let ft_face: &freetype::Face = font
                    .ft_face()
                    .unwrap_or_else(|| panic!("not foun face:{:?}", font.name()));
                let cairo_font_face = FontFace::create_from_ft(ft_face).unwrap();

                self.context.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                self.context.set_font_face(&cairo_font_face);

                for con in text.text_items() {
                    let tj = (-con.adjust() * 0.001) * font_size * horz_scale;
                    let mrm = Matrix::new_translation_matrix(tj, 0.0);
                    text_matrix = mrm.mutiply(&text_matrix);

                    let chars = font.decode_chars(con.bytes());
                    for char in chars.iter() {
                        let mut displacement =
                            font.get_char_width(char) * 0.001 * font_size + char_spacing;
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
                        let x = user_matrix.v31;
                        let y = user_matrix.v32;

                        let scale_x = user_matrix.v11;
                        let scale_y = user_matrix.v22;
                        let scale = ((scale_y * scale_y + scale_x * scale_x) * 0.5).sqrt();
                        self.context.set_font_size(scale);
                        if let Some(gid) = font.glyph_index_from_charcode(char) {
                            let g = Glyph::new(gid as u64, x, y);
                            let glyphs = vec![g];
                            self.context.show_glyphs(glyphs.as_slice()).unwrap();
                        } else {
                            println!("gid is not found");
                        }
                        if font.is_vertical() {
                            let trm = Matrix::new_translation_matrix(0.0, displacement);
                            text_matrix = trm.mutiply(&text_matrix);
                        } else {
                            let mrm = Matrix::new_translation_matrix(displacement, 0.0);
                            text_matrix = mrm.mutiply(&text_matrix);
                        }
                        // move
                    }
                }
            }
            GraphicsObject::Image(image) => {
                let w = image.width()?;
                let h = image.height()?;
                let trm = Matrix::new(1.0 / w, 0.0, 0.0, 1.0 / h, 0.0, 1.0);
                let userctm = trm.mutiply(image.ctm());
                let ctm = userctm.mutiply(&self.ctm);

                let x = ctm.v31;
                let y = ctm.v32;
                let rgb_iamge = image.rgb_image()?;

                let mut im = RgbImage::new(w as u32, h as u32);
                let mut data = Vec::new();
                for i in 0..(h as u32) {
                    for j in 0..(w as u32) {
                        let index = i * (w as u32) + j;
                        let pixel: &ColorRGBValue = rgb_iamge.get(index as usize).unwrap();
                        data.push(pixel.r());
                        data.push(pixel.g());
                        data.push(pixel.b());
                        data.push(0);
                        im.put_pixel(j, i, Rgb([pixel.r(), pixel.g(), pixel.b()]));
                    }
                }
                // im.save("test.png").unwrap();
                //let mut bytes = Vec::new();
                // TODO other image format
                let stride = Format::Rgb24.stride_for_width(w as u32).unwrap();
                let i_s =
                    ImageSurface::create_for_data(data, Format::Rgb24, w as i32, h as i32, stride)
                        .unwrap();
                self.context.set_source_surface(i_s, x, y).unwrap();
                self.context.paint().unwrap();
            }
        }
        Ok(())
    }
}
