use cairo::{Context, Format, ImageSurface};
use freetype::Face;
use pdf::{
    device::Device,
    error::Result,
    font::{CharCode, GlyphDesc},
    geom::{coordinate::Matrix, path::Path, sub_path::PathSegment},
    page::{
        graphics_state::{FillRule, GraphicsState, TextRenderingMode},
        image::PdfImage,
    },
};
pub struct CairoDevice {
    context: Context,
    surface: ImageSurface,
    width: f32,
    height: f32,
    face: Option<Face>,
    hdpi: f32,
    vdpi: f32,
    page_num: u32,
}

impl CairoDevice {
    pub fn new(hdpi: f32, vdpi: f32) -> Self {
        let surface = ImageSurface::create(Format::ARgb32, 600, 600).unwrap();
        let context = Context::new(&surface).unwrap();
        CairoDevice {
            surface,
            context,
            width: 0.0,
            height: 0.0,
            face: None,
            hdpi,
            vdpi,
            page_num: 0,
        }
    }

    fn set_matrix(&self, ctm: &Matrix) {
        self.context.set_matrix(cairo::Matrix::new(
            ctm.a as f64,
            ctm.b as f64,
            ctm.c as f64,
            ctm.d as f64,
            ctm.e as f64,
            ctm.f as f64,
        ));
    }
}

impl Device for CairoDevice {
    fn clip(&mut self, state: &GraphicsState) -> Result<()> {
        let ctm = &state.ctm;
        self.set_matrix(ctm);
        let path = &state.clipping_path;
        self.context.new_path();
        for sb in path.subpaths() {
            for seg in sb.segments() {
                match seg {
                    PathSegment::MoveTo(p) => self.context.move_to(p.x() as f64, p.y() as f64),
                    PathSegment::LineTo(p) => self.context.line_to(p.x() as f64, p.y() as f64),
                    PathSegment::Curve3(c3) => {
                        let p0 = &c3.p0;
                        let p1 = &c3.p1;
                        let p2 = &c3.p2;
                        self.context.curve_to(
                            p0.x() as f64,
                            p0.y() as f64,
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                        );
                    }
                    PathSegment::Curve4(c4) => {
                        let p1 = &c4.p1;
                        let p2 = &c4.p2;
                        let p3 = &c4.p3;
                        self.context.curve_to(
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                            p3.x() as f64,
                            p3.y() as f64,
                        );
                    }
                    PathSegment::Closed => {
                        self.context.close_path();
                    }
                }
            }
        }
        self.context.close_path();
        self.context.clip();
        return Ok(());
    }
    fn start_page(
        &mut self,
        state: &GraphicsState,
        page_num: u32,
        width: f32,
        height: f32,
    ) -> pdf::error::Result<()> {
        self.surface =
            ImageSurface::create(Format::ARgb32, width.ceil() as i32, height.ceil() as i32)
                .unwrap();
        self.context = Context::new(&self.surface).unwrap();
        self.context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        self.context.paint().unwrap();
        self.width = width;
        self.height = height;
        self.page_num = page_num;

        Ok(())
    }

    fn hdpi(&self) -> f32 {
        self.hdpi
    }

    fn vdpi(&self) -> f32 {
        self.vdpi
    }

    fn draw_char(&mut self, char: &CharCode, state: &GraphicsState) -> Result<()> {
        let font = &state.font.as_ref().unwrap();
        if font.fontfile().is_none() {
            return Ok(());
        }
        match state.render_mode {
            TextRenderingMode::INVisible => {
                return Ok(());
            }
            _ => {}
        }
        let face = self.face.as_ref().unwrap();
        let glyph = font.get_glyph(char).unwrap();
        let cs = &state.fill_color_space;
        let cv = &state.fill_color_value;
        let rgb = cs.rgb(cv)?;
        // TODO why?
        let ox = -char.origin_x() * 0.001 * state.font_size;
        let oy = -char.origin_y() * 0.001 * state.font_size;

        let fm = Matrix::new_translation_matrix(ox, oy);
        let x = 0.0;
        let y = 0.0;
        let ct = fm.transform(&state.text_matrix).transform(&state.ctm);
        //let ct = state.text_matrix.transform(&state.ctm);

        self.context.save().unwrap();
        self.context.identity_matrix();
        self.set_matrix(&ct);
        self.context.scale(1.0, -1.0);
        self.context.set_font_size(state.font_size as f64);
        self.context
            .set_source_rgb(rgb.r as f64, rgb.g as f64, rgb.b as f64);

        match &glyph {
            GlyphDesc::Name(n) => {
                let gid = face.get_name_index(n.as_str()).unwrap();
                let g = cairo::Glyph::new(gid as u64, x, y);
                let glyphs = vec![g];
                self.context.show_glyphs(glyphs.as_slice()).unwrap();
            }
            GlyphDesc::Gid(gid) => {
                let g = cairo::Glyph::new(gid.to_owned() as u64, x, y);
                let glyphs = vec![g];
                self.context.show_glyphs(glyphs.as_slice()).unwrap();
            }
        }
        self.context.restore().unwrap();
        Ok(())
    }

    fn end_text(&mut self, state: &GraphicsState) -> Result<()> {
        Ok(())
    }

    fn end_page(&mut self, state: &GraphicsState) -> Result<()> {
        let mut file = std::fs::File::create(format!("page{}_.png", self.page_num)).unwrap();
        self.surface.write_to_png(&mut file).unwrap();
        Ok(())
    }

    fn draw_image(&mut self, image: PdfImage, state: &GraphicsState) -> Result<()> {
        let w = image.width();
        let h = image.height();
        let ctm = &state.ctm;
        let img_ctm = Matrix::new(ctm.a, ctm.b, -ctm.c, -ctm.d, ctm.c + ctm.e, ctm.d + ctm.f);

        self.context.save().unwrap();
        self.context.identity_matrix();
        self.set_matrix(&img_ctm);
        self.context.scale((1.0 / w) as f64, (1.0 / h) as f64);
        if image.is_mask() {
            // TODO how to fix this
            let mut data = image.image_data().to_vec();
            if (w * h) as usize == data.len() {
                let stride = Format::A8.stride_for_width(w as u32).unwrap();
                let pad = ((image.width() as usize + 3) & (!3)) - image.width() as usize;
                let mut pad_data = vec![0; pad];
                let image_data: Vec<u8> = data
                    .chunks_mut(w as usize)
                    .map(|line| [line, &mut pad_data].concat())
                    .flatten()
                    .collect();
                let m_s = ImageSurface::create_for_data(
                    image_data,
                    Format::A8,
                    w as i32,
                    h as i32,
                    stride,
                )
                .unwrap();
                let scont = Context::new(&m_s).unwrap();
                scont.set_source_rgba(1.0, 1.0, 1.0, 0.0);
                scont.paint().unwrap();
                self.context.set_source_rgba(1.0, 1.0, 0.0, 0.0);
                self.context.mask_surface(m_s, 0.0, 0.0).unwrap();
            } else {
                let stride = Format::ARgb32.stride_for_width(w as u32).unwrap();
                let mut rgb_data = Vec::new();
                let rgb_image = image.rgb_image()?;
                for j in 0..h as usize {
                    for i in 0..w as usize {
                        let rgb = &rgb_image[j * (w as usize) + i];
                        rgb_data.push((rgb.b * 255.0) as u8);
                        rgb_data.push((rgb.g * 255.0) as u8);
                        rgb_data.push((rgb.r * 255.0) as u8);
                        rgb_data.push(255);
                    }
                }
                let i_s = ImageSurface::create_for_data(
                    rgb_data,
                    Format::ARgb32,
                    w as i32,
                    h as i32,
                    stride,
                )
                .unwrap();
                self.context.set_source_surface(i_s, 0.0, 0.0).unwrap();
            }
        } else {
            let stride = Format::ARgb32.stride_for_width(w as u32).unwrap();
            let mut rgb_data = Vec::new();
            let rgb_image = image.rgb_image()?;
            for j in 0..h as usize {
                for i in 0..w as usize {
                    let rgb = &rgb_image[j * (w as usize) + i];
                    rgb_data.push((rgb.b * 255.0) as u8);
                    rgb_data.push((rgb.g * 255.0) as u8);
                    rgb_data.push((rgb.r * 255.0) as u8);
                    rgb_data.push(255);
                }
            }
            let i_s =
                ImageSurface::create_for_data(rgb_data, Format::ARgb32, w as i32, h as i32, stride)
                    .unwrap();
            self.context.set_source_surface(i_s, 0.0, 0.0).unwrap();
        }

        self.context.paint().unwrap();
        self.context.restore().unwrap();
        self.context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        Ok(())
    }

    fn fill_path(&mut self, path: &Path, state: &GraphicsState, rule: FillRule) -> Result<()> {
        let cs = &state.fill_color_space;
        let cv = &state.fill_color_value;
        let rgb = cs.rgb(cv)?;
        self.context.save().unwrap();
        self.context.identity_matrix();
        self.set_matrix(&state.ctm);
        self.context
            .set_source_rgb(rgb.r as f64, rgb.g as f64, rgb.b as f64);
        match rule {
            FillRule::Winding => {
                self.context.set_fill_rule(cairo::FillRule::Winding);
            }
            FillRule::EvenOdd => {
                self.context.set_fill_rule(cairo::FillRule::EvenOdd);
            }
        }
        for sp in path.subpaths() {
            for seg in sp.segments() {
                match seg {
                    PathSegment::MoveTo(p) => {
                        self.context.move_to(p.x() as f64, p.y() as f64);
                    }
                    PathSegment::LineTo(p) => {
                        self.context.line_to(p.x() as f64, p.y() as f64);
                    }
                    PathSegment::Curve3(c3) => {
                        let p0 = &c3.p0;
                        let p1 = &c3.p1;
                        let p2 = &c3.p2;
                        self.context.curve_to(
                            p0.x() as f64,
                            p0.y() as f64,
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                        );
                    }
                    PathSegment::Curve4(c4) => {
                        //let p0 = self.cairo_point(&c4.p0);
                        let p1 = &c4.p1;
                        let p2 = &c4.p2;
                        let p3 = &c4.p3;
                        self.context.curve_to(
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                            p3.x() as f64,
                            p3.y() as f64,
                        );
                    }
                    PathSegment::Closed => {
                        self.context.close_path();
                    }
                }
            }
            self.context.close_path();
        }
        self.context.fill().unwrap();
        self.context.restore().unwrap();
        Ok(())
    }

    fn begin_text(&mut self, state: &GraphicsState) -> pdf::error::Result<()> {
        Ok(())
    }

    fn update_font(&mut self, state: &GraphicsState) -> pdf::error::Result<()> {
        let font = state.font.as_ref().unwrap();
        if let Some(program) = font.fontfile() {
            let program = program.to_vec();
            let library = freetype::Library::init().unwrap();
            let face = library.new_memory_face(program, 0).unwrap();
            let ctm = state.text_matrix.transform(&state.ctm);
            let width = state.font_size * ctm.a;
            let height = state.font_size * ctm.d;
            face.set_pixel_sizes(width as u32, height as u32).unwrap();
            let cairo_face = cairo::FontFace::create_from_ft(&face).unwrap();
            self.context.set_font_face(&cairo_face);
            self.face = Some(face);
        } else {
            println!("Fontfile is None");
        }
        Ok(())
    }

    fn stroke_path(&mut self, path: &Path, state: &GraphicsState) -> pdf::error::Result<()> {
        let cs = &state.stroke_color_space;
        let cv = &state.stroke_color_value;
        let rgb = cs.rgb(cv)?;
        self.context.save().unwrap();
        self.context.identity_matrix();
        self.set_matrix(&state.ctm);
        self.context
            .set_source_rgb(rgb.r as f64, rgb.g as f64, rgb.b as f64);
        let lw = state.line_width as f64;
        self.context.set_line_width(lw);

        for sp in path.subpaths() {
            for seg in sp.segments() {
                match seg {
                    pdf::geom::sub_path::PathSegment::MoveTo(p) => {
                        self.context.move_to(p.x() as f64, p.y() as f64);
                    }
                    pdf::geom::sub_path::PathSegment::LineTo(p) => {
                        self.context.line_to(p.x() as f64, p.y() as f64);
                    }
                    pdf::geom::sub_path::PathSegment::Curve3(c3) => {
                        let p0 = &c3.p0;
                        let p1 = &c3.p1;
                        let p2 = &c3.p2;
                        self.context.curve_to(
                            p0.x() as f64,
                            p0.y() as f64,
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                        );
                    }
                    pdf::geom::sub_path::PathSegment::Curve4(c4) => {
                        let p0 = &c4.p0;
                        self.context.move_to(p0.x() as f64, p0.y() as f64);
                        let p1 = &c4.p1;
                        let p2 = &c4.p2;
                        let p3 = &c4.p3;
                        self.context.curve_to(
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                            p3.x() as f64,
                            p3.y() as f64,
                        );
                    }
                    PathSegment::Closed => {
                        self.context.close_path();
                    }
                }
            }
            self.context.close_path();
        }
        self.context.stroke().unwrap();
        self.context.restore().unwrap();
        Ok(())
    }

    fn fill_and_stroke_path(
        &mut self,
        path: &Path,
        state: &GraphicsState,
        rule: FillRule,
    ) -> Result<()> {
        let fill_cs = &state.fill_color_space;
        let fill_cv = &state.fill_color_value;
        let fill_rgb = fill_cs.rgb(fill_cv)?;
        self.context.save().unwrap();
        self.context.identity_matrix();
        self.set_matrix(&state.ctm);
        self.context
            .set_source_rgb(fill_rgb.r as f64, fill_rgb.g as f64, fill_rgb.b as f64);
        match rule {
            FillRule::Winding => {
                self.context.set_fill_rule(cairo::FillRule::Winding);
            }
            FillRule::EvenOdd => {
                self.context.set_fill_rule(cairo::FillRule::EvenOdd);
            }
        }
        for sp in path.subpaths() {
            for seg in sp.segments() {
                match seg {
                    PathSegment::MoveTo(p) => {
                        self.context.move_to(p.x() as f64, p.y() as f64);
                    }
                    PathSegment::LineTo(p) => {
                        self.context.line_to(p.x() as f64, p.y() as f64);
                    }
                    PathSegment::Curve3(c3) => {
                        let p0 = &c3.p0;
                        let p1 = &c3.p1;
                        let p2 = &c3.p2;
                        self.context.curve_to(
                            p0.x() as f64,
                            p0.y() as f64,
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                        );
                    }
                    PathSegment::Curve4(c4) => {
                        //let p0 = self.cairo_point(&c4.p0);
                        let p1 = &c4.p1;
                        let p2 = &c4.p2;
                        let p3 = &c4.p3;
                        self.context.curve_to(
                            p1.x() as f64,
                            p1.y() as f64,
                            p2.x() as f64,
                            p2.y() as f64,
                            p3.x() as f64,
                            p3.y() as f64,
                        );
                    }
                    PathSegment::Closed => {
                        self.context.close_path();
                    }
                }
            }
            self.context.close_path();
        }
        self.context.fill_preserve().unwrap();
        let stroke_cs = &state.stroke_color_space;
        let stroke_cv = &state.stroke_color_value;
        let stroke_rgb = stroke_cs.rgb(&stroke_cv)?;
        self.context.set_source_rgb(
            stroke_rgb.r as f64,
            stroke_rgb.g as f64,
            stroke_rgb.b as f64,
        );
        self.context.stroke().unwrap();
        self.context.restore().unwrap();
        Ok(())
    }
}
