use crate::color::ColorSpace;

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct GeneralState {
    pub blend_mode: Option<String>,
    pub strok_alpha: f64,
    pub fill_alpha: f64,
    pub render_indent: f64,
    pub stroke_adjust: bool,
    pub alpha_source: bool,
    pub fill_color: ColorSpace,
    pub stroke_color: ColorSpace,
}
