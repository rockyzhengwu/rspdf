use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::matrix::Matrix;
use crate::geom::rectangle::Rectangle;
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

    pub fn finish_page(&mut self) {
        self.xml.push_str("</page>");
    }
}

impl Device for TraceDevice {
    fn start_page(&mut self, num: u32, bbox: Rectangle) {
        self.xml.push_str(
            format!(
                "<page number={},x={} y={} width={} height={}>\n",
                num,
                bbox.lx(),
                bbox.ly(),
                bbox.width(),
                bbox.height()
            )
            .as_str(),
        );
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
                let ctm = text.ctm();
                self.xml.push_str(
                    format!(
                        "<text ctm=\"{} {} {} {} {} {}\" font=\"{}\">\n",
                        ctm.v11,
                        ctm.v12,
                        ctm.v21,
                        ctm.v22,
                        ctm.v31,
                        ctm.v32,
                        font.name()
                    )
                    .as_str(),
                );
                for con in text.text_items() {
                    let unicode = font.to_unicode(con.bytes());
                    let tj = (-con.adjust() * 0.001) * font_size * horz_scale;
                    let mrm = Matrix::new_translation_matrix(tj, 0.0);
                    text_matrix = mrm.mutiply(&text_matrix);
                    let chars = font.decode_chars(con.bytes());
                    for (i, char) in chars.iter().enumerate() {
                        let u = unicode.get(i).map_or("", |x| x);
                        let mut displacement =
                            font.get_char_width(char) * 0.001 * font_size + char_spacing;
                        if char.is_space() {
                            displacement += word_spacing;
                        }
                        let gid = font.glyph_index_from_charcode(char).map_or(0, |x| x);
                        self.xml.push_str(
                            format!(
                                "<g unicode=\"{}\" glyph=\"{}\" x=\"{}\" y= \"{}\">\n",
                                u, gid, text_matrix.v31, text_matrix.v32
                            )
                            .as_str(),
                        );
                        if font.is_vertical() {
                            let trm = Matrix::new_translation_matrix(0.0, displacement);
                            text_matrix = trm.mutiply(&text_matrix);
                        } else {
                            let mrm = Matrix::new_translation_matrix(displacement, 0.0);
                            text_matrix = mrm.mutiply(&text_matrix);
                        }
                    }
                }
                self.xml.push_str("</text>\n")
            }
            GraphicsObject::Image(_) => {}
            GraphicsObject::Path(p) => {
                println!("path: {:?}", p);
            }
        }
        Ok(())
    }
}
