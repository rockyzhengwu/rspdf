use crate::canvas::matrix::Matrix;
use crate::canvas::{path_info::PathInfo, text_info::TextInfo};
use crate::device::Device;
use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;

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
    fn begain_page(&mut self, page_num: u32, media: &Rectangle, crop: &Rectangle) {
        self.result().push_str(
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
        for line in &self.lines {
            let s = line.string();
            self.results.push(format!("<textline>\n{}</textline>", s));
        }
        self.results.push("</page>".to_string());
    }

    fn show_text(&mut self, textinfo: &mut TextInfo) -> PDFResult<()> {
        let cids = textinfo.cids();
        let content = textinfo.get_unicode(cids.as_slice());
        let cids = textinfo.cids();
        let scale = 1.0;
        let mut chars = content.chars();
        let ctm = Matrix::default();
        for cid in &cids {
            match textinfo.get_glyph(cid, &scale) {
                Some(glyph) => {
                    let (x, y) = textinfo.out_pos(cid, &ctm);
                    let mut item = TextItem::default();
                    let bitmap = glyph.bitmap();
                    let font = textinfo.font().to_string();
                    let font_size = textinfo.font_size();
                    item.font = font;
                    item.font_size = font_size;
                    item.unicode = chars.next().unwrap();
                    item.bbox =
                        Rectangle::new(x, y, x + bitmap.width() as f64, y + bitmap.rows() as f64);
                    let mut ch = String::new();
                    ch.push(item.unicode);
                    self.add_text_item(item);
                }
                None => {
                    panic!("bitmap is NOne");
                }
            }
        }

        Ok(())
    }

    fn paint_path(&mut self, _pathinfo: PathInfo) -> PDFResult<()> {
        Ok(())
    }
}
