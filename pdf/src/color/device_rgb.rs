
#[derive(Debug)]
pub struct DeviceRGB {
    r: f32,
    g: f32,
    b: f32,
}

impl Default for DeviceRGB {
    fn default() -> Self {
        DeviceRGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

impl DeviceRGB {
    pub fn set_rgb(&mut self, r: f32, g: f32, b: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
    }
}
