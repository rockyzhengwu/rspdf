use std::fmt::Display;

use crate::{
    color::{device_gray::DeviceGray, value::ColorValue, ColorSpace},
    error::{PdfError, Result},
    font::pdf_font::Font,
    geom::coordinate::Matrix,
    geom::path::Path,
    object::PdfObject,
};

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

#[derive(Debug, Clone)]
pub enum FillRule {
    Winding,
    EvenOdd,
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

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
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

impl Display for TextRenderingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextRenderingMode::Fill => write!(f, "Fill"),
            TextRenderingMode::Stroke => write!(f, "Stroke"),
            TextRenderingMode::FillStroke => write!(f, "FillStroke"),
            TextRenderingMode::INVisible => write!(f, "INVisible"),
            TextRenderingMode::FillClip => write!(f, "FillClip"),
            TextRenderingMode::StrokeClip => write!(f, "StrokeClip"),
            TextRenderingMode::FillStrokeClip => write!(f, "FillStrokeClip"),
            TextRenderingMode::Clip => write!(f, "Clip"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum RenderIntent {
    AbsoluteColorimetric,
    #[default]
    RelativeColorimetric,
    Saturation,
    Perceptual,
}

impl RenderIntent {
    pub fn new_from_str(name: &str) -> Result<Self> {
        match name {
            "AbsoluteColorimetric" => Ok(RenderIntent::AbsoluteColorimetric),
            "RelativeColorimetric" => Ok(RenderIntent::RelativeColorimetric),
            "Saturation" => Ok(RenderIntent::Saturation),
            "Perceptual" => Ok(RenderIntent::Perceptual),
            _ => return Err(PdfError::Interpreter("invalid RenderIntent ".to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphicsState {
    // device-indepdent
    pub ctm: Matrix,
    pub clipping_path: Path,
    pub fill_color_space: ColorSpace,
    pub fill_color_value: ColorValue,
    pub stroke_color_space: ColorSpace,
    pub stroke_color_value: ColorValue,

    pub line_width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
    pub dash_pattern: DashPattern,
    pub stroke_adjust: bool,
    pub render_intent: RenderIntent,
    pub blend_mode: BlendMode,
    pub soft_mask: Option<PdfObject>,
    pub stroke_alpha: f32,
    pub alpha_constant: f32,
    pub alpha_source: bool,
    pub black_point_compensatioin: PdfObject,

    // device depdent
    pub fill_overprint: bool,
    pub stroke_overprint: bool,
    pub overpint_mode: i32,
    pub black_generation: PdfObject,
    // undercolor_removal
    // transfer
    // halftone
    // flatness
    pub smoothness: f32,
    pub font_size: f32,
    pub char_space: f32,
    pub word_space: f32,
    pub render_mode: TextRenderingMode,
    pub font: Option<Font>,
    pub text_rise: f32,
    pub text_horz_scale: f32,
    pub text_leading: f32,
    pub text_matrix: Matrix,
    pub text_line_matrix: Matrix,
    pub flatness: i32,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::default(),
            clipping_path: Path::default(),
            fill_color_space: ColorSpace::DeviceGray(DeviceGray::new()),
            stroke_color_space: ColorSpace::DeviceGray(DeviceGray::new()),
            fill_color_value: ColorValue::default(),
            stroke_color_value: ColorValue::default(),
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
            black_point_compensatioin: PdfObject::Null,
            fill_overprint: false,
            stroke_overprint: false,
            overpint_mode: 1,
            black_generation: PdfObject::Null,
            smoothness: 0.0,
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
            flatness: 0,
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
        self.ctm = mat.transform(&self.ctm);
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

    pub fn set_line_cap(&mut self, cap: i32) {
        match cap {
            0 => self.line_cap = LineCap::Butt,
            1 => self.line_cap = LineCap::Round,
            2 => self.line_cap = LineCap::PjectingSquare,
            _ => {}
        }
    }

    pub fn set_line_join(&mut self, join: i32) {
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

    pub fn set_render_mode(&mut self, mode: i32) {
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

    pub fn text_horz_scale(&self) -> f32 {
        if self.text_horz_scale == 0.0 {
            1.0
        } else {
            self.text_horz_scale * 0.01
        }
    }
}
