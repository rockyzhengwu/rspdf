use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::matrix::Matrix;
use crate::geom::point::Point;
use crate::geom::rectangle::Rectangle;
use crate::page::graphics_object::GraphicsObject;

#[derive(Default, Clone)]
pub struct TextDevice {
    lines: Vec<String>,
    current: String,
    last_pos: Point,
}

impl TextDevice {
    pub fn new() -> Self {
        TextDevice {
            lines: Vec::new(),
            current: String::new(),
            last_pos: Point::default(),
        }
    }

    pub fn result(&self) -> String {
        self.lines.join("\n")
    }
    pub fn is_same_line(&self, x: f64, y: f64, font_size: f64) -> bool {
        let dw = x - self.last_pos.x();
        let dy = y - self.last_pos.y();
        dy == 0.0 && dw < font_size
    }
}

impl Device for TextDevice {
    fn start_page(&mut self, _num: u32, _bbox: Rectangle) {
        self.lines.clear();
        self.current.clear();
        self.last_pos = Point::default();
    }
    fn process(&mut self, obj: &GraphicsObject) -> PDFResult<()> {
        match obj {
            GraphicsObject::Text(text) => {
                let font = text.font();
                let font_size = text.font_size();
                let mut text_matrix = text.text_matrix().to_owned();
                let char_spacing = text.char_spacing();
                let horz_scale = text.text_horz_scale();
                let word_spacing = text.word_space();
                for con in text.text_items() {
                    let unicode = font.to_unicode(con.bytes());
                    let tj = (-con.adjust() * 0.001) * font_size * horz_scale;
                    let mrm = Matrix::new_translation_matrix(tj, 0.0);
                    text_matrix = mrm.mutiply(&text_matrix);
                    if self.current.is_empty()
                        || self.is_same_line(text_matrix.v31, text_matrix.v32, font_size)
                    {
                        self.current.push_str(unicode.join("").to_string().as_str());
                    } else {
                        self.lines.push(std::mem::take(&mut self.current));
                    }
                    let chars = font.decode_chars(con.bytes());
                    let mut displacement = 0.0;
                    for char in chars.iter() {
                        displacement +=
                            font.get_char_width(char) * 0.001 * font_size + char_spacing;
                        if char.is_space() {
                            displacement += word_spacing;
                        }
                    }
                    if font.is_vertical() {
                        let trm = Matrix::new_translation_matrix(0.0, displacement);
                        text_matrix = trm.mutiply(&text_matrix);
                    } else {
                        let mrm = Matrix::new_translation_matrix(displacement, 0.0);
                        text_matrix = mrm.mutiply(&text_matrix);
                    }
                }
                self.last_pos = Point::new(text_matrix.v31, text_matrix.v32);
            }
            GraphicsObject::Path(_path) => {}
            _ => {}
        }
        Ok(())
    }
}
