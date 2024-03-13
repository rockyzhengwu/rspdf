use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;
use crate::page::graphics_object::GraphicsObject;

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
struct TextItem {
    font: String,
    font_size: f64,
    char_spacing: f64,
    word_spacing: f64,
    unicode: char,
    binary: Vec<u8>,
    bbox: Rectangle,
}

impl TextItem {
    fn is_same_line(&self, other: &TextItem) -> bool {
        self.bbox.ly() == other.bbox.ly()
    }
}

#[derive(Default, Clone, Debug)]
struct TextLine {
    items: Vec<TextItem>,
}

impl TextLine {
    pub fn add(&mut self, item: &TextItem) -> bool {
        if let Some(last) = self.items.last_mut() {
            if last.is_same_line(item) {
                self.items.push(item.to_owned());
                return true;
            }
        }
        self.items.push(item.to_owned());
        false
    }
    pub fn string(&self) -> String {
        let mut line = String::new();
        for item in &self.items {
            let x = item.bbox.lx();
            let y = item.bbox.ly();
            let u = item.unicode;
            let s = format!("<item x={x} y={y}>{u}</item>\n");
            line.push_str(s.as_str())
        }
        line
    }
}

#[derive(Default, Clone)]
pub struct TextDevice {
    results: Vec<String>,
    lines: Vec<TextLine>,
}

// TODO add textoption
impl TextDevice {
    pub fn new() -> Self {
        TextDevice {
            results: vec!["<document>".to_string()],
            lines: Vec::new(),
        }
    }

    pub fn result(&self) -> String {
        let mut s = self.results.join("\n");
        s.push_str("</document>");
        s
    }

    fn add_text_item(&mut self, item: TextItem) {
        if let Some(l) = self.lines.last_mut() {
            if l.add(&item) {
                return;
            }
        }
        let mut line = TextLine::default();
        line.add(&item);
        self.lines.push(line);
    }
}

impl Device for TextDevice {
    fn start_page(&mut self, bbox:Rectangle) {
        
    }
    fn process(&mut self, obj: &GraphicsObject) -> PDFResult<()> {
        println!("{:?}", obj);
        Ok(())
    }
}
