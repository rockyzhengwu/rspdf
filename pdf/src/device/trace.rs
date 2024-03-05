use crate::device::Device;
use crate::errors::PDFResult;
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
    fn process(&mut self, obj: &GraphicsObject) -> PDFResult<()> {
        match obj {
            GraphicsObject::Text(text) => {
                let font = text.font();
                for code in text.char_codecs() {
                    let chu = font.unicode(code);
                    print!("{:?} ", chu)
                }
                println!(" ");
            }
            GraphicsObject::Image(image) => {
                println!("image, {:?}", image);
            }
            GraphicsObject::Path(path) => {
                println!("path:{:?}", path);
            }
        }
        Ok(())
    }
}
