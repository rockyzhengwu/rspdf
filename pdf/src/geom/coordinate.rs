#[derive(Debug, Clone)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix {
    pub fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Matrix { a, b, c, d, e, f }
    }

    pub fn transform(&self, right: &Matrix) -> Matrix {
        let a = self.a * right.a + self.b * right.c;
        let b = self.a * right.b + self.b * right.d;
        let c = self.c * right.a + self.d * right.c;
        let d = self.c * right.b + self.d * right.d;
        let e = self.e * right.a + self.f * right.c + right.e;
        let f = self.e * right.b + self.f * right.d + right.f;
        Matrix { a, b, c, d, e, f }
    }

    pub fn new_translation_matrix(e: f32, f: f32) -> Matrix {
        Matrix::new(1.0, 0.0, 0.0, 1.0, e, f)
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn transform(&self, matrix: &Matrix) -> Point {
        let x = matrix.a * self.x + matrix.c * self.y + matrix.e;
        let y = matrix.b * self.x + matrix.c * self.y + matrix.e;
        Point { x, y }
    }
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }
}
