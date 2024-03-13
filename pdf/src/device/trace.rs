use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::matrix::Matrix;
use crate::page::graphics_object::GraphicsObject;

#[derive(Default, Clone)]
pub struct TraceDevice {
    xml: String,
}

impl TraceDevice {
    pub fn new(path: &str) -> Self {
        let mut xml = String::new();
        xml.push_str(format!("<document path=\"{}\">\n", path).as_str());
        TraceDevice { xml }
    }
}
impl TraceDevice {
    pub fn result(&mut self) -> &str {
        self.xml.push_str("</document>");
        self.xml.as_str()
    }
}

impl Device for TraceDevice {
    fn start_page(&mut self, bbox: crate::geom::rectangle::Rectangle) {}
    fn process(&mut self, obj: &GraphicsObject) -> PDFResult<()> {
        match obj {
            GraphicsObject::Text(text) => {
                let font = text.font();
                let font_size = text.font_size();
                let mut text_matrix = text.text_matrix().to_owned();
                let char_spacing = text.char_spacing();
                let horz_scale = text.text_horz_scale();
                let word_spacing = text.word_space();
                let text_rise = text.text_rise();
                for con in text.text_items() {
                    let unicode = font.to_unicode(con.bytes());
                    let tj = (-con.adjust() * 0.001) * font_size * horz_scale;
                    let mrm = Matrix::new_translation_matrix(tj, 0.0);
                    text_matrix = mrm.mutiply(&text_matrix);

                    let chars = font.decode_chars(con.bytes());
                    for (i, char) in chars.iter().enumerate() {
                        let u = unicode.get(i);
                        println!("{:?},{:?},{:?}", u, text_matrix.v31, text_matrix.v32);
                        let mut displacement = (font.get_char_width(char) * 0.001) * horz_scale;
                        if char.is_space() {
                            displacement += word_spacing;
                        }

                        let m = Matrix::new_translation_matrix(displacement, 0.0);
                        text_matrix.v31 += displacement;
                        // text_matrix = m.mutiply(&text_matrix);
                    }
                }
            }
            GraphicsObject::Image(_) => {}
            GraphicsObject::Path(_) => {}
        }
        Ok(())
    }
}
