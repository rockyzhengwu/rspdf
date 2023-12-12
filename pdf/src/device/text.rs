use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::canvas::{path_info::PathInfo, text_info::TextInfo};
use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;

#[derive(Default, Clone)]
pub struct TextDevice {
    output: PathBuf,
    current: String,
    results: Vec<String>,
}
// todo add textoption

impl TextDevice {
    pub fn new(output: PathBuf) -> Self {
        TextDevice {
            output,
            current: "<document>".to_string(),
            results: Vec::new(),
        }
    }
}

impl Device for TextDevice {
    fn begain_page(&mut self, media: &Rectangle, crop: &Rectangle) {
        self.current = String::new();
        self.current.push_str(
            format!(
                "<page mediabox=\"{},{},{},{}\" cropbox=\"{},{},{},{}\">",
                media.x(),
                media.y(),
                media.width(),
                media.height(),
                crop.x(),
                crop.y(),
                crop.width(),
                crop.height(),
            )
            .as_str(),
        );
    }

    fn end_page(&mut self) {
        self.current.push_str("</page>");
        let text = std::mem::take(&mut self.current);
        println!("{:?}", text);
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

    fn close(&mut self) -> PDFResult<()> {
        self.current.push_str("</document>");
        let mut file = File::create(self.output.as_path()).unwrap();
        file.write_all(self.results.join("\n").as_bytes()).unwrap();
        Ok(())
    }
}
