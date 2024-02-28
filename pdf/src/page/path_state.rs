// Table 54 – Line Cap Styles
#[derive(Debug, Clone)]
pub enum LineCap {
    Butt,
    Round,
    ProjectingSquare,
}

// Table 55 – Line Join Styles (continued)
#[derive(Debug, Clone)]
pub enum LineJoinStyle {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone)]
pub struct PathState {
    line_cap: LineCap,
    line_join: LineJoinStyle,
    dash_phase: f64,
    miter_limit: f64,
    line_width: f64,
    dash_array: Vec<f64>,
}

impl Default for PathState {
    fn default() -> Self {
        PathState {
            line_cap: LineCap::Butt,
            line_join: LineJoinStyle::Miter,
            dash_phase: 0.0,
            miter_limit: 10.0,
            line_width: 0.0,
            dash_array: Vec::new(),
        }
    }
}
impl PathState {
    pub fn set_line_cap(&mut self, cap: i64) {
        match cap {
            0 => self.line_cap = LineCap::Butt,
            1 => self.line_cap = LineCap::Round,
            2 => self.line_cap = LineCap::ProjectingSquare,
            _ => {}
        }
    }

    pub fn set_line_join(&mut self, join: i64) {
        match join {
            0 => self.line_join = LineJoinStyle::Miter,
            1 => self.line_join = LineJoinStyle::Round,
            2 => self.line_join = LineJoinStyle::Bevel,
            _ => {}
        }
    }

    pub fn set_dash_phase(&mut self, phase: f64) {
        self.dash_phase = phase;
    }

    pub fn set_miter_limit(&mut self, limit: f64) {
        self.miter_limit = limit;
    }
    pub fn set_line_width(&mut self, width: f64) {
        self.line_width = width;
    }
    pub fn set_dash_array(&mut self, array: Vec<f64>) {
        self.dash_array = array
    }
}
