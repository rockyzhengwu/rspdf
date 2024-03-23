
#[derive(Debug)]
pub struct DeviceCMYK {
    c: f32,
    m: f32,
    y: f32,
    k: f32,
}

impl Default for DeviceCMYK {
    fn default() -> Self {
        DeviceCMYK {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 0.0,
        }
    }
}

impl DeviceCMYK {
    pub fn set_cmyk(&mut self, c: f32, m: f32, y: f32, k: f32) {
        self.c = c;
        self.m = m;
        self.y = y;
        self.k = k;
    }
}
