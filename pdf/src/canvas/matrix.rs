#[derive(Debug, Clone)]
pub struct Matrix {
    pub v11: f64,
    pub v12: f64,
    pub v13: f64,
    pub v21: f64,
    pub v22: f64,
    pub v23: f64,
    pub v31: f64,
    pub v32: f64,
    pub v33: f64,
}

impl Matrix {
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Matrix {
            v11: a,
            v12: b,
            v13: 0.0,
            v21: c,
            v22: d,
            v23: 0.0,
            v31: e,
            v32: f,
            v33: 1.0,
        }
    }

    pub fn new_translation_matrix(x: f64, y: f64) -> Matrix {
        Matrix {
            v11: 1.0,
            v12: 0.0,
            v13: 0.0,
            v21: 0.0,
            v22: 1.0,
            v23: 0.0,
            v31: x,
            v32: y,
            v33: 1.0,
        }
    }

    pub fn mutiply(&self, by: &Matrix) -> Matrix {
        Matrix {
            v11: self.v11 * by.v11 + self.v12 * by.v21 + self.v13 * by.v31,
            v12: self.v11 * by.v12 + self.v12 * by.v22 + self.v13 * by.v32,
            v13: self.v11 * by.v13 + self.v12 * by.v23 + self.v13 * by.v33,
            v21: self.v21 * by.v11 + self.v22 * by.v21 + self.v23 * by.v31,
            v22: self.v21 * by.v12 + self.v22 * by.v22 + self.v23 * by.v32,
            v23: self.v21 * by.v13 + self.v22 * by.v23 + self.v23 * by.v33,
            v31: self.v31 * by.v11 + self.v32 * by.v21 + self.v33 * by.v31,
            v32: self.v31 * by.v12 + self.v32 * by.v22 + self.v33 * by.v32,
            v33: self.v31 * by.v13 + self.v32 * by.v23 + self.v33 * by.v33,
        }
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Matrix {
            v11: 1.0,
            v12: 0.0,
            v13: 0.0,
            v21: 0.0,
            v22: 1.0,
            v23: 0.0,
            v31: 0.0,
            v32: 0.0,
            v33: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;

    #[test]
    fn test_translation() {
        let old = Matrix::new(13.9183, 0.0, 0.0, 13.9183, 36.7398, 608.9446);
        let tl = Matrix::new_translation_matrix(3.1968, 0.0);
        let new = tl.mutiply(&old);
        assert_eq!(new.v31, 81.23382144000001);
    }
}
