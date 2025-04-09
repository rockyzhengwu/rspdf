use std::fmt::Write;

use crate::{
    device::Device,
    error::Result,
    font::{CharCode, GlyphDesc},
    geom::{coordinate::Matrix, path::Path, sub_path::PathSegment},
    page::{
        graphics_state::{FillRule, GraphicsState, TextRenderingMode},
        image::PdfImage,
    },
};

pub struct TextWord {
    x: f32,
    y: f32,
    unicode: String,
    glyph: GlyphDesc,
}

impl TextWord {
    pub fn new(x: f32, y: f32, unicode: String, glyph: GlyphDesc) -> Self {
        Self {
            x,
            y,
            unicode,
            glyph,
        }
    }
    pub fn xml(&self) -> String {
        format!(
            "<g unicode=\"{}\" glyph=\"{}\" x=\"{}\" y=\"{}\" />\n",
            self.unicode, self.glyph, self.x, self.y
        )
    }
}

pub struct TextSpan {
    font: String,
    font_size: f32,
    render_mode: TextRenderingMode,
    words: Vec<TextWord>,
}

impl TextSpan {
    pub fn new(font: String, font_size: f32, render_mode: TextRenderingMode) -> Self {
        TextSpan {
            font,
            font_size,
            render_mode,
            words: Vec::new(),
        }
    }
    pub fn add_word(&mut self, word: TextWord) {
        self.words.push(word)
    }

    pub fn xml(&self) -> String {
        let mut res = String::new();
        res.push_str(
            format!(
                "<text_span font=\"{}\" font_size=\"{}\" render_mode=\"{}\"> \n",
                self.font, self.font_size, self.render_mode
            )
            .as_str(),
        );
        for word in self.words.iter() {
            res.push_str(word.xml().as_str());
        }
        res.push_str("</text_span>\n");
        res
    }
}

pub struct TextBlock {
    spans: Vec<TextSpan>,
}

impl TextBlock {
    pub fn new() -> Self {
        TextBlock { spans: Vec::new() }
    }
    pub fn xml(&self) -> String {
        let mut res = String::new();
        res.push_str("<text_block>\n");
        for span in self.spans.iter() {
            res.push_str(span.xml().as_str());
        }
        res.push_str("</text_block>\n");
        res
    }

    pub fn add_textspan(&mut self, font: &str, font_size: f32, render_mode: TextRenderingMode) {
        if let Some(last) = self.spans.last() {
            if last.font == font && last.font_size == font_size {
                return;
            }
        }
        let span = TextSpan::new(font.to_string(), font_size, render_mode);
        self.spans.push(span)
    }

    pub fn last_font(&self) -> Option<&str> {
        if let Some(span) = self.spans.last() {
            return Some(span.font.as_str());
        }
        None
    }
    pub fn add_textword(
        &mut self,
        font: &str,
        font_size: f32,
        render_mode: TextRenderingMode,
        word: TextWord,
    ) {
        if let Some(last) = self.spans.last_mut() {
            // TODO consider writing mode and if word position is in line
            if last.font.as_str() == font
                && last.font_size == font_size
                && last.render_mode == render_mode
            {
                last.add_word(word);
                return;
            }
        }
        let mut span = TextSpan::new(font.to_string(), font_size, render_mode);
        span.add_word(word);
        self.spans.push(span);
    }
    pub fn clear(&mut self) {
        self.spans.clear()
    }
}

pub struct Trace {
    text_block: TextBlock,
    content: String,
    ctm: Matrix,
}

impl Trace {
    pub fn new() -> Self {
        Trace {
            text_block: TextBlock::new(),
            content: String::new(),
            ctm: Matrix::default(),
        }
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }
}

impl Device for Trace {
    fn start_page(
        &mut self,
        state: &GraphicsState,
        page_num: u32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        self.text_block = TextBlock::new();
        self.content.clear();
        self.content
            .push_str(format!("<page page_num=\"{}\">\n", page_num).as_str());
        self.ctm = state.ctm.clone();
        Ok(())
    }

    fn end_page(&mut self, state: &GraphicsState) -> Result<()> {
        self.content.push_str("</page>");
        Ok(())
    }

    fn draw_char(&mut self, char: &CharCode, state: &GraphicsState) -> Result<()> {
        let font = state.font.as_ref().unwrap();
        let font_size = state.font_size;
        let tm = &state.text_matrix;
        let unicode = font.unicode(char).unwrap();
        let glyph = font.get_glyph(char).unwrap();
        let ox = -char.origin_x() * 0.001 * font_size;
        let oy = -char.origin_y() * 0.001 * font_size;
        let font_matrix = Matrix::new(1.0, 0.0, 0.0, 1.0, ox, oy);
        let tm = font_matrix.transform(&tm);

        let x = tm.e;
        let y = tm.f;
        let word = TextWord::new(x, y, unicode, glyph);
        let render_mode = state.render_mode.to_owned();
        self.text_block
            .add_textword(font.name(), font_size, render_mode, word);

        Ok(())
    }

    fn begin_text(&mut self, state: &GraphicsState) -> Result<()> {
        self.text_block = TextBlock::new();
        Ok(())
    }

    fn end_text(&mut self, state: &GraphicsState) -> Result<()> {
        self.content
            .write_str(self.text_block.xml().as_str())
            .unwrap();
        self.text_block.clear();
        Ok(())
    }

    fn draw_image(&mut self, image: PdfImage, state: &GraphicsState) -> Result<()> {
        if image.is_mask() {
            return Ok(());
        }
        let ctm = &state.ctm;
        if let Some(cs) = image.color_space() {
            self.content.push_str(
                format!(
                    "<image width=\"{}\" height=\"{}\" colorspace=\"{}\" transform=\"{} {} {} {} {} {}\" />\n",
                    image.width(),
                    image.height(),
                    cs,
                    ctm.a,
                    ctm.b,
                    ctm.c,
                    ctm.d,
                    ctm.e,
                    ctm.f
                )
                .as_str(),
            );
        } else {
            self.content.push_str(
                format!(
                    "<image width=\"{}\" height=\"{}\" colorspace=\"{}\" />",
                    image.width(),
                    image.height(),
                    "None",
                )
                .as_str(),
            );
        }
        Ok(())
    }
    fn hdpi(&self) -> f32 {
        72.0
    }

    fn vdpi(&self) -> f32 {
        72.0
    }

    fn fill_path(&mut self, path: &Path, state: &GraphicsState, rule: FillRule) -> Result<()> {
        let ctm = &state.ctm;
        let colorspace = &state.fill_color_space;
        self.content.push_str(
            format!(
                "<fill_path colorspace=\"{}\" transform=\"{} {} {} {} {} {}\" >\n",
                colorspace, ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f
            )
            .as_str(),
        );
        for subpath in path.subpaths() {
            for seg in subpath.segments() {
                match seg {
                    PathSegment::MoveTo(p) => {
                        self.content.push_str(
                            format!("<moveto x=\"{}\" y=\"{}\"/>\n", p.x(), p.y()).as_str(),
                        );
                    }
                    PathSegment::LineTo(p) => {
                        self.content.push_str(
                            format!("<lineto x=\"{}\" y=\"{}\"/>\n", p.x(), p.y()).as_str(),
                        );
                    }
                    PathSegment::Curve3(bz) => {
                        self.content.push_str(
                            format!(
                                "<bezier_quad x0=\"{}\" y0=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />\n", 
                                bz.p0.x(), bz.p0.y(), bz.p1.x(), bz.p1.y(), bz.p2.x(), bz.p2.y()).as_str(),
                        );
                    }
                    PathSegment::Curve4(bz) => {
                        self.content.push_str(
                            format!(
                                "<bezier_cubic x0=\"{}\" y0=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" x3=\"{}\" y3=\"{}\"  />\n", 
                                bz.p0.x(), bz.p0.y(), bz.p1.x(), bz.p1.y(), bz.p2.x(), bz.p2.y(), bz.p3.x(),bz.p3.y()).as_str(),
                        );
                    }
                    _ => {}
                }
            }
        }
        self.content.push_str("</fill_path>\n");
        Ok(())
    }
    fn update_font(&mut self, state: &GraphicsState) -> Result<()> {
        Ok(())
    }

    fn stroke_path(&mut self, path: &Path, state: &GraphicsState) -> Result<()> {
        let ctm = &state.ctm;
        let colorspace = &state.fill_color_space;

        self.content.push_str(
            format!(
                "<stroke_path colorspace=\"{}\" transform=\"{} {} {} {} {} {}\" >\n",
                colorspace, ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f
            )
            .as_str(),
        );
        for subpath in path.subpaths() {
            for seg in subpath.segments() {
                match seg {
                    PathSegment::MoveTo(p) => {
                        self.content.push_str(
                            format!("<moveto x=\"{}\" y=\"{}\"/>\n", p.x(), p.y()).as_str(),
                        );
                    }
                    PathSegment::LineTo(p) => {
                        self.content.push_str(
                            format!("<lineto x=\"{}\" y=\"{}\"/>\n", p.x(), p.y()).as_str(),
                        );
                    }
                    PathSegment::Curve3(bz) => {
                        self.content.push_str(
                            format!(
                                "<bezier_quad x0=\"{}\" y0=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />\n",
                                bz.p0.x(), bz.p0.y(), bz.p1.x(), bz.p1.y(), bz.p2.x(), bz.p2.y()).as_str(),
                        );
                    }
                    PathSegment::Curve4(bz) => {
                        self.content.push_str(
                            format!(
                                "<bezier_cubic x0=\"{}\" y0=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" x3=\"{}\" y3=\"{}\"  />\n",
                                bz.p0.x(), bz.p0.y(), bz.p1.x(), bz.p1.y(), bz.p2.x(), bz.p2.y(), bz.p3.x(),bz.p3.y()).as_str(),
                        );
                    }
                    PathSegment::Closed => self.content.push_str("<closepath />\n"),
                }
            }
        }
        self.content.push_str("</stroke_path>\n");
        Ok(())
    }

    fn fill_and_stroke_path(
        &mut self,
        path: &Path,
        state: &GraphicsState,
        rule: FillRule,
    ) -> Result<()> {
        Ok(())
    }
}
