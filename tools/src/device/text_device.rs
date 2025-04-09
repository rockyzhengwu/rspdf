use pdf::{
    device::Device, error::Result, font::CharCode, geom::coordinate::Matrix,
    page::graphics_state::GraphicsState,
};

#[derive(Default)]
pub struct TextDevice {
    page_num: u32,
    last_x: f32,
    last_y: f32,
    lines: Vec<String>,
    current_line: String,
}
impl TextDevice {
    pub fn page_content(&self) -> String {
        self.lines.join("\n")
    }
}

impl Device for TextDevice {
    fn start_page(
        &mut self,
        _state: &GraphicsState,
        page_num: u32,
        _width: f32,
        _height: f32,
    ) -> pdf::error::Result<()> {
        self.page_num = page_num;
        self.lines.clear();
        self.current_line.clear();
        Ok(())
    }

    fn draw_char(&mut self, char: &CharCode, state: &GraphicsState) -> Result<()> {
        if let Some(font) = &state.font {
            let unicode = font.unicode(char)?;
            let font_size = state.font_size;
            let ox = -char.origin_x() * 0.001 * font_size;
            let oy = -char.origin_y() * 0.001 * font_size;
            let font_matrix = Matrix::new(1.0, 0.0, 0.0, 1.0, ox, oy);
            let tm = font_matrix.transform(&state.text_matrix);
            let x = tm.e;
            let y = tm.f;
            let width = char.width() * 0.01;
            if self.current_line.is_empty() {
                self.current_line.push_str(unicode.as_str());
            } else if !self.current_line.is_empty() && y == self.last_y {
                let dx = x - self.last_x;
                if dx > 0.0 && dx > width * 3.0 {
                    self.current_line.push(' ');
                }
                self.current_line.push_str(unicode.as_str());
            } else {
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                    self.current_line.push_str(unicode.as_str());
                }
            }
            self.last_x = x;
            self.last_y = y;
        }
        Ok(())
    }
    fn end_page(&mut self, _state: &GraphicsState) -> Result<()> {
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
        }
        Ok(())
    }
}
