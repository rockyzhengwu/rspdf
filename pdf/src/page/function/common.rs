#[derive(Debug, Clone)]
pub struct CommonFunction {
    domain: Vec<f32>,
    range: Option<Vec<f32>>,
    input_number: usize,
    output_number: usize,
}

impl CommonFunction {
    pub fn new(domain: Vec<f32>, range: Option<Vec<f32>>) -> Self {
        let input_number = domain.len() / 2;
        let output_number = range.as_ref().map_or(0, |v| v.len() / 2);

        CommonFunction {
            domain,
            range,
            input_number,
            output_number,
        }
    }

    pub fn domain(&self) -> &[f32] {
        self.domain.as_slice()
    }

    pub fn range(&self) -> Option<&[f32]> {
        self.range.as_deref()
    }

    pub fn input_number(&self) -> usize {
        self.input_number
    }

    pub fn output_number(&self) -> usize {
        self.output_number
    }

    pub fn get_domain(&self, index: usize) -> &f32 {
        self.domain.get(index).unwrap()
    }
    pub fn get_range(&self, index: usize) -> &f32 {
        // TODO fix this
        self.range.as_ref().unwrap().get(index).unwrap()
    }
}
