use crate::canvas::{path_info::PathInfo, text_info::TextInfo};
use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;

#[derive(Default, Clone)]
struct TextItem{
    font: String,
    font_size: f64,
    char_spacing: f64,
    word_spacing: f64,
    unicode: String,
    binary: Vec<u8>,
    bbox: Rectangle,
}

impl TextItem {
    pub fn merge(&mut self, other: TextItem) {
        unimplemented!()
    }
}

#[derive(Default, Clone)]
pub struct TextDevice {
    current: String,
    results: Vec<String>,
}
// TODO add textoption

impl TextDevice {
    pub fn new() -> Self {
        TextDevice {
            current: String::new(),
            results: vec!["<document>".to_string()],
        }
    }
    pub fn result(&self) -> String {
        let mut s = self.results.join("\n");
        s.push_str("</document>");
        s
    }
}

impl Device for TextDevice {
    fn begain_page(&mut self, page_num: u32, media: &Rectangle, crop: &Rectangle) {
        self.current = String::new();
        self.current.push_str(
            format!(
                "<page={} mediabox=\"{},{},{},{}\" cropbox=\"{},{},{},{}\">",
                page_num,
                media.lx(),
                media.ly(),
                media.ux(),
                media.uy(),
                crop.lx(),
                crop.ly(),
                crop.ux(),
                crop.uy(),
            )
            .as_str(),
        );
    }

    fn end_page(&mut self, _page_num: u32) {
        self.current.push_str("</page>");
        let text = std::mem::take(&mut self.current);
        self.results.push(text);
    }

    fn show_text(&mut self, textinfo: TextInfo) -> PDFResult<()> {
        let content = textinfo.get_unicode();
        let (x, y) = textinfo.position();
        let s = format!("<textspan x={x}, y={y}>{content}</textpan>\n");
        self.current.push_str(s.as_str());
        Ok(())
    }

    fn paint_path(&mut self, _pathinfo: PathInfo) -> PDFResult<()> {
        Ok(())
    }
}
