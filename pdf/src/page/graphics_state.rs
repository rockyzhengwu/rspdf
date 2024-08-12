use crate::geom::matrix::Matrix;
use crate::geom::rectangle::Rectangle;
use crate::object::{PDFDictionary, PDFObject};
use crate::color::ColorSpace;
use crate::font::pdf_font::Font;

#[derive(Default, Debug, Clone)]
pub struct DashPattern{
    array: Vec<usize>,
    pharse: usize,
}

#[derive(Default, Debug, Clone)]
pub enum LineCap{
    #[default]
    Butt,
    Round,
    PjectingSquare,
}

#[derive(Default, Debug, Clone)]
pub enum LineJoin{
    #[default]
    Miter,
    Round,
    Bevel,
}

// TODO finish this
#[derive(Default, Debug, Clone)]
pub enum BlendMode{
    #[default]
    Normal,
}

#[derive(Debug, Clone, Default)]
pub enum TextRenderingMode {
    #[default]
    Fill,
    Stroke,
    FillStroke,
    INVisible,
    FillClip,
    StrokeClip,
    FillStrokeClip,
    Clip,
}


#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
pub struct GraphicsState {
    // device-indepdent
    pub(crate) ctm: Matrix,
    pub(crate) clipping_path: Rectangle,
    pub(crate) color_space: ColorSpace,
    pub(crate) fill_color: ColorSpace,
    pub(crate) stroke_color: ColorSpace,
    pub(crate) line_width:  f64,
    pub(crate) line_cap: LineCap,
    pub(crate) line_join: LineJoin,
    pub(crate) miter_limit: f64,
    pub(crate) dash_pattern: DashPattern,
    pub(crate) stroke_adjust: bool,
    pub(crate) render_indent: f64,
    pub(crate) blend_mode: BlendMode,
    pub(crate) soft_mask: Option<PDFObject>,
    pub(crate) strok_alpha: f64,
    pub(crate) alpha_constant: f64,
    pub(crate) alpha_source: bool,
    pub(crate) black_point_compensatioin: PDFObject,

    // device depdent
    pub(crate) overprint:bool ,
    pub(crate) overpint_mode: usize,
    pub(crate) black_generation: PDFObject,
    // undercolor_removal
    // transfer
    // halftone
    // flatness
    // smoothness
    //
    //
   
    pub(crate) font_size: f64,
    pub(crate) char_space: f64,
    pub(crate) word_space: f64,
    pub(crate) render_mode: TextRenderingMode,
    pub(crate) font: Option<Font>,
    pub(crate) text_rise: f64,
    pub(crate) text_horz_scale: f64,
    pub(crate) text_leading: f64,
    pub(crate) text_matrix: Matrix,
    pub(crate) text_line_matrix: Matrix,

}

impl GraphicsState {
    pub fn new(ctm: Matrix) -> Self {
        GraphicsState {
            ctm,
            ..Default::default()
        }
    }
    pub fn update_ctm_matrix(&mut self, mat: &Matrix) {
        self.ctm = mat.mutiply(&self.ctm);
    }

    pub fn process_ext_gs(&mut self, _obj: PDFDictionary) {
        unimplemented!()
    }

    pub fn set_line_cap(&mut self, cap: i64) {
        match cap {
            0 => self.line_cap = LineCap::Butt,
            1 => self.line_cap = LineCap::Round,
            2 => self.line_cap = LineCap::PjectingSquare,
            _ => {}
        }
    }

    pub fn set_line_join(&mut self, join: i64) {
        match join {
            0 => self.line_join = LineJoin::Miter,
            1 => self.line_join = LineJoin::Round,
            2 => self.line_join = LineJoin::Bevel,
            _ => {}
        }
    }

    pub fn set_dash_phase(&mut self, phase: f64) {
        unimplemented!()
    }

    pub fn set_dash_array(&mut self, array: Vec<f64>) {
        unimplemented!()
    }

    pub fn font(&self) -> &Font {
        self.font.as_ref().unwrap()
    }

    pub fn set_render_mode(&mut self, mode: i64) {
        match mode {
            0 => self.render_mode = TextRenderingMode::Fill,
            1 => self.render_mode = TextRenderingMode::Stroke,
            2 => self.render_mode = TextRenderingMode::FillStroke,
            3 => self.render_mode = TextRenderingMode::INVisible,
            4 => self.render_mode = TextRenderingMode::FillClip,
            5 => self.render_mode = TextRenderingMode::StrokeClip,
            6 => self.render_mode = TextRenderingMode::FillStrokeClip,
            7 => self.render_mode = TextRenderingMode::Clip,
            _ => {}
        }
    }

    pub fn text_horz_scale(&self) -> f64 {
        if self.text_horz_scale == 0.0 {
            1.0
        } else {
            self.text_horz_scale * 0.01
        }
    }
}
