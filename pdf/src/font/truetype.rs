use std::io::{Read, Seek};

use crate::document::Document;
use crate::errors::PDFResult;
use crate::font::ft_font::FTFont;
use crate::geom::rectangle::Rectangle;
use crate::object::PDFObject;

pub struct TrueTypeFont {
    name: Option<String>,
    flags: i32,
    stemv: i32,
    ascent: i32,
    descent: i32,
    italic_angle: i32,
    char_width: [i32; 256],
    glphy_index: [u32; 256],
    char_bbox: [Rectangle; 256],
    font_bbox: Rectangle,
    ft_font: FTFont,
}

impl TrueTypeFont {
    pub fn is_ttot(&self) -> bool {
        self.ft_font.is_ttot()
    }
}

impl Default for TrueTypeFont {
    fn default() -> Self {
        TrueTypeFont {
            name: None,
            flags: 0,
            stemv: 0,
            ascent: 0,
            descent: 0,
            italic_angle: 0,
            char_width: [0; 256],
            glphy_index: [0; 256],
            char_bbox: [Rectangle::default(); 256],
            font_bbox: Rectangle::default(),
            ft_font: FTFont::default(),
        }
    }
}

fn load_font_descriptor(desc: &PDFObject, font: &mut TrueTypeFont) -> PDFResult<()> {
    if let Some(flags) = desc.get_value_as_i32("Flags") {
        font.flags = flags?;
    }
    if let Some(italic_angle) = desc.get_value_as_i32("ItalicAngle") {
        font.italic_angle = italic_angle?;
    }
    if let Some(stemv) = desc.get_value_as_i32("StemV") {
        font.stemv = stemv?;
    }

    if let Some(ascent) = desc.get_value_as_i32("Ascent") {
        font.ascent = ascent?;
    }
    if let Some(descent) = desc.get_value_as_i32("Descent") {
        font.descent = descent?;
    }

    if let Some(PDFObject::Arrray(values)) = desc.get_value("FontBBox") {
        let lx = values[0].as_f64()?;
        let ly = values[1].as_f64()?;
        let ux = values[2].as_f64()?;
        let uy = values[3].as_f64()?;
        let rectangle = Rectangle::new(lx, ly, ux, uy);
        font.font_bbox = rectangle;
    }
    if let Some(missing_width) = desc.get_value_as_i32("MissingWidth") {
        let width = missing_width?;
        font.char_width = [width; 256];
    }
    Ok(())
}

fn load_width(font: &mut TrueTypeFont, obj: &PDFObject) -> PDFResult<()> {
    if let Some(widths) = obj.get_value("Widths") {
        let first_char = obj.get_value("FirstChar").unwrap().as_i32()?;
        let last_char = obj.get_value("LastChar").unwrap().as_i32()?;
        let ws = widths.as_array()?;
        if first_char > 255 {
            return Ok(());
        }
        for i in first_char..=last_char {
            let index = (i - first_char) as usize;
            if let Some(v) = ws.get(index) {
                font.char_width[i as usize] = v.as_i32()?;
            }
        }
        return Ok(());
    }
    Ok(())
}

fn load_encoding(obj: &PDFObject, font: &mut TrueTypeFont) {
    let encoding = obj.get_value("Encoding");
    match encoding{
        None=>{
            // set default encoing
        }
        Some(PDFObject::Name(_))=>{

        }
        Some(PDFObject::Dictionary(_))=>{

        }
        _=>{}
    }
}

pub fn create_truetype_font<T: Seek + Read>(
    fontname: &str,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<TrueTypeFont> {
    let mut font = TrueTypeFont::default();
    if let Some(name_res) = obj.get_value_as_string("BaseFont") {
        let name = name_res?;
        font.name = Some(name.to_string());
    }
    if let Some(descriptor) = obj.get_value("FontDescriptor") {
        let desc = doc.get_object_without_indriect(descriptor)?;
        load_font_descriptor(&desc, &mut font)?;
        let ff = desc.get_value("FontFile");
        let ff2 = desc.get_value("FontFile2");
        let ff3 = desc.get_value("FontFile3");
        let fontfile = ff.or(ff2).or(ff3);
        if let Some(program) = fontfile {
            let ft_font = FTFont::try_new(program.bytes()?)?;
            font.ft_font = ft_font;
        }
    }
    load_width(&mut font, obj)?;
    if !font.ft_font.is_loaded() {
        // TODO load builtin font
    }
    // LoadEncoding
    // note must after load descriptor
    unimplemented!()
}
