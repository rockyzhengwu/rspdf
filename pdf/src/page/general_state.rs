#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct GeneralState {
    blend_mode: Option<String>,
    strok_alpha: f64,
    fill_alpha: f64,
    render_indent: f64,
    stroke_adjust: bool,
    alpha_source: bool,
}
