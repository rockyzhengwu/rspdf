use crate::errors::PDFResult;
use crate::geom::rectangle::Rectangle;
use crate::object::PDFObject;

#[derive(Clone, Debug, Default)]
pub struct FontDescriptor {
    flags: Option<i32>,
    italic_angle: f64,
    ascent: f64,
    descent: f64,
    cap_height: f64,
    x_height: f64,
    missing_width: f64,
    stem_v: f64,
    bbox: Rectangle,
    embed: Option<PDFObject>,
}

impl FontDescriptor {
    pub fn is_symbolic(&self) -> bool {
        if let Some(f) = &self.flags {
            return f & 4 == 0;
        }
        false
    }

    pub fn new_from_object(desc: &PDFObject) -> PDFResult<Self> {
        let mut d = FontDescriptor::default();
        if let Some(flags) = desc.get_value_as_i32("Flags") {
            d.flags = Some(flags?);
        }

        if let Some(ascent) = desc.get_value_as_f64("Ascent") {
            d.ascent = ascent?;
        }
        if let Some(cap_height) = desc.get_value_as_f64("CapHeight") {
            d.cap_height = cap_height?;
        }
        if let Some(x_height) = desc.get_value_as_f64("XHeight") {
            d.x_height = x_height?;
        }
        if let Some(descent) = desc.get_value_as_f64("Descent") {
            d.descent = descent?;
        }
        if let Some(missing_width) = desc.get_value_as_f64("MissingWidth") {
            d.missing_width = missing_width?;
        }
        if let Some(italic_angle) = desc.get_value_as_f64("ItalicAngle") {
            d.italic_angle = italic_angle?;
        }
        if let Some(stem_v) = desc.get_value_as_f64("StemV") {
            d.stem_v = stem_v?;
        }
        if let Some(PDFObject::Arrray(values)) = desc.get_value("FontBBox") {
            let lx = values[0].as_f64()?;
            let ly = values[1].as_f64()?;
            let ux = values[2].as_f64()?;
            let uy = values[3].as_f64()?;
            let rectangle = Rectangle::new(lx, ly, ux, uy);
            d.bbox = rectangle;
        }
        let ff = desc.get_value("FontFile");
        let ff2 = desc.get_value("FontFile2");
        let ff3 = desc.get_value("FontFile3");
        let program = ff.or(ff2).or(ff3);
        d.embed = program.map(|x| x.to_owned());
        Ok(d)
    }

    pub fn is_embeded(&self) -> bool {
        self.embed.is_some()
    }

    pub fn embeded(&self) -> Option<&PDFObject> {
        self.embed.as_ref()
    }
}
