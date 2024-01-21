use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::{path::Path, rectangle::Rectangle};
use crate::page::text::PageText;

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
    fn begain_page(&mut self, page_num: &u32, _media: Option<Rectangle>, _crop: Option<Rectangle>) {
        let s = format!("<page number={}>\n", page_num);
        self.xml.push_str(s.as_str());
    }

    fn end_page(&mut self, _page_num: &u32) {
        self.xml.push_str("</page>");
    }

    fn paint_path(&mut self, path: &Path) -> PDFResult<()> {
        println!("{:?}", path);
        Ok(())
    }
    fn start_text(&mut self) {
        self.xml.push_str("<text_span>\n");
    }

    fn show_text(&mut self, textobj: &PageText) -> PDFResult<()> {
        for item in textobj.items() {
            self.xml.push_str(
                format!(
                    "<text_item lx=\"{}\" ly=\"{}\">{}</text_item>\n",
                    item.bbox().lx(),
                    item.bbox().ly(),
                    item.unicode(),
                )
                .as_str(),
            )
        }
        Ok(())
    }
    fn end_text(&mut self) {
        self.xml.push_str("</text_span>\n");
    }
}
