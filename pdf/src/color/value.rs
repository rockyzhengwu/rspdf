#[derive(Debug, Clone)]
pub struct ColorValue {
    values: Vec<f32>,
}

impl ColorValue {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }
    pub fn values(&self) -> &[f32] {
        self.values.as_slice()
    }
    pub fn value_size(&self) -> usize {
        self.values.len()
    }
}

impl Default for ColorValue {
    fn default() -> Self {
        Self { values: vec![0.0] }
    }
}

#[derive(Debug, Clone)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl ColorRgb {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        ColorRgb { r, g, b }
    }
}
