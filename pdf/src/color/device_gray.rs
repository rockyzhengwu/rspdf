#[derive(Debug, Clone)]
pub struct DeviceGray {
    gray: f32,
}
impl Default for DeviceGray {
    fn default() -> Self {
        DeviceGray { gray: 0.0 }
    }
}
impl DeviceGray {
    pub fn set_gray(&mut self, gray: f32) {
        self.gray = gray;
    }
}
