#[derive(Debug, Clone)]
pub struct CommonFunction {
    domain: Vec<f32>,
    range: Option<Vec<f32>>,
}

impl CommonFunction {
    pub fn new(domain: Vec<f32>, range: Option<Vec<f32>>) -> Self {
        CommonFunction { domain, range }
    }

    pub fn domain(&self) -> &[f32] {
        self.domain.as_slice()
    }

    pub fn range(&self) -> Option<&[f32]> {
        self.range.as_deref()
    }
}
