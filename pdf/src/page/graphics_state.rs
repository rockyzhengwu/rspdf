use crate::color::{ColorSpace, ColorValue};
use crate::errors::PDFResult;
use crate::font::pdf_font::Font;
use crate::geom::matrix::Matrix;
use crate::geom::path::Path;
use crate::object::PDFObject;

#[derive(Default, Debug, Clone)]
pub struct DashPattern {
    array: Vec<u32>,
    phase: u32,
}
impl DashPattern {
    pub fn new(array: Vec<u32>, phase: u32) -> Self {
        DashPattern { array, phase }
    }
    pub fn array(&self) -> &[u32] {
        self.array.as_slice()
    }

    pub fn phase(&self) -> u32 {
        self.phase
    }
}

#[derive(Default, Debug, Clone)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    PjectingSquare,
}

#[derive(Default, Debug, Clone)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

// TODO finish this
#[derive(Default, Debug, Clone)]
pub enum BlendMode {
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

#[derive(Debug, Clone, Default)]
pub enum RenderIntent {
    AbsoluteColorimetric,
    #[default]
    RelativeColorimetric,
    Saturation,
    Perceptual,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphicsState {
    // device-indepdent
    pub(crate) ctm: Matrix,
    pub(crate) clipping_path: Path,
    pub(crate) color_space: Option<ColorSpace>,
    pub(crate) fill_color_space: Option<ColorSpace>,
    pub(crate) stroke_color_space: Option<ColorSpace>,
    pub(crate) fill_color_value: Option<ColorValue>,
    pub(crate) stroke_color_value: Option<ColorValue>,

    pub(crate) line_width: f64,
    pub(crate) line_cap: LineCap,
    pub(crate) line_join: LineJoin,
    pub(crate) miter_limit: f64,
    pub(crate) dash_pattern: DashPattern,
    pub(crate) stroke_adjust: bool,
    pub(crate) render_intent: RenderIntent,
    pub(crate) blend_mode: BlendMode,
    pub(crate) soft_mask: Option<PDFObject>,
    pub(crate) stroke_alpha: f64,
    pub(crate) alpha_constant: f64,
    pub(crate) alpha_source: bool,
    pub(crate) black_point_compensatioin: PDFObject,

    // device depdent
    pub(crate) overprint: bool,
    pub(crate) overpint_mode: u32,
    pub(crate) black_generation: PDFObject,
    // undercolor_removal
    // transfer
    // halftone
    // flatness
    // smoothness
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

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::default(),
            clipping_path: Path::default(),
            color_space: None,
            fill_color_space: None,
            stroke_color_space: None,
            fill_color_value: None,
            stroke_color_value: None,

            line_width: 1.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            miter_limit: 10.0,
            dash_pattern: DashPattern::default(),
            stroke_adjust: false,
            render_intent: RenderIntent::RelativeColorimetric,
            blend_mode: BlendMode::default(),
            soft_mask: None,
            stroke_alpha: 1.0,
            alpha_constant: 1.0,
            alpha_source: false,
            black_point_compensatioin: PDFObject::default(),
            overprint: false,
            overpint_mode: 1,
            black_generation: PDFObject::default(),
            font_size: 0.0, // no default value font_size
            char_space: 0.0,
            word_space: 0.0,
            render_mode: TextRenderingMode::default(),
            font: None,
            text_rise: 0.0,
            text_horz_scale: 0.0,
            text_leading: 0.0,
            text_matrix: Matrix::default(),
            text_line_matrix: Matrix::default(),
        }
    }
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

    pub fn set_render_intent(&mut self, intent: &str) {
        match intent {
            "AbsoluteColorimetric" => self.render_intent = RenderIntent::AbsoluteColorimetric,
            "RelativeColorimetric" => self.render_intent = RenderIntent::RelativeColorimetric,
            "Saturation" => self.render_intent = RenderIntent::Saturation,
            "Perceptual" => self.render_intent = RenderIntent::Perceptual,
            _ => {}
        }
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

    pub fn set_dash_pattern(&mut self, array: Vec<u32>, phase: u32) {
        let dash = DashPattern::new(array, phase);
        self.dash_pattern = dash;
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

    pub fn update_by_extgstate(&mut self, state: &PDFObject) -> PDFResult<()> {
        if let Some(lw) = state.get_value("LW") {
            let lw = lw.as_f64()?;
            self.line_width = lw;
        }
        if let Some(lc) = state.get_value("LC") {
            let lc = lc.as_i64()?;
            self.set_line_cap(lc);
        }
        if let Some(lj) = state.get_value("LJ") {
            let lj = lj.as_i64()?;
            self.set_line_join(lj);
        }
        if let Some(ml) = state.get_value("ML") {
            let ml = ml.as_f64()?;
            self.miter_limit = ml;
        }
        if let Some(d) = state.get_value("D") {
            let d = d.as_array()?;
            let dash_array = d.first().unwrap().as_array()?;
            let mut array = Vec::new();
            for v in dash_array {
                array.push(v.as_u32()?);
            }
            let phase = &d.last().unwrap().as_u32()?;
            self.set_dash_pattern(array, phase.to_owned());
        }
        if let Some(ri) = state.get_value("RI") {
            let intent = ri.as_string()?;
            self.set_render_intent(&intent);
        }
        if let Some(op) = state.get_value("OP") {
            let op = op.as_bool()?;
            self.overprint = op;
        }

        if let Some(op) = state.get_value("op") {
            let op = op.as_bool()?;
            self.overprint = op;
        }

        if let Some(op) = state.get_value("OPM") {
            let op = op.as_u32()?;
            self.overpint_mode = op;
        }

        if let Some(smask) = state.get_value("SMask") {
            // sofmtmask
        }

        // TODO more op

        Ok(())
    }
}
