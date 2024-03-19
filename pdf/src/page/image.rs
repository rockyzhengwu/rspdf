use crate::geom::matrix::Matrix;
use crate::page::graphics_state::GraphicsState;

#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct Image {
    width: f64,
    height: f64,
    bits_per_component: u32,
    data: Vec<u8>,
    graphics_state: GraphicsState,
}

impl Image {
    pub fn new(
        width: f64,
        height: f64,
        bits_per_component: u32,
        data: Vec<u8>,
        graphics_state: GraphicsState,
    ) -> Self {
        Image {
            width,
            height,
            bits_per_component,
            data,
            graphics_state,
        }
    }
    pub fn ctm(&self) -> &Matrix {
        self.graphics_state.ctm()
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn bits_per_component(&self) -> u32 {
        self.bits_per_component
    }
}
