use std::collections::HashMap;

use crate::error::{PdfError, Result};
use crate::font::descriptor::Descriptor;
use crate::font::font_program::FontProgram;
use crate::font::CharCode;
use crate::object::dictionary::PdfDict;
use crate::object::{array::PdfArray, PdfObject};
use crate::xref::Xref;

#[derive(Debug, Default, Clone)]
pub struct CidFont {
    sub_type: String,
    base_font: String,
    w2: Option<HashMap<u32, (f32, f32, f32)>>,
    w: Option<HashMap<u32, f32>>,
    dw: Option<f32>,
    dw2: Option<(f32, f32)>,
    descriptor: Option<Descriptor>,
    program: Option<FontProgram>,
}

impl CidFont {
    pub fn base_font(&self) -> &str {
        self.base_font.as_str()
    }
    pub fn try_new(dict: &PdfDict, xref: &Xref) -> Result<Self> {
        let mut font = CidFont::default();
        let sub_type = dict
            .get("Subtype")
            .ok_or(PdfError::Font("CidFont Subtype is need".to_string()))?;
        let sub_type = sub_type.as_name()?.name();
        let base_font = dict
            .get("BaseFont")
            .ok_or(PdfError::Font("CidFont Basefont is need".to_string()))?
            .as_name()?
            .name();
        font.sub_type = sub_type.to_string();
        font.base_font = base_font.to_string();
        if let Some(dw) = dict.get("DW") {
            let d = dw
                .as_number()
                .map_err(|_| PdfError::Font("CidFont Dw is not a number".to_string()))?
                .real();
            font.dw = Some(d);
        }
        if let Some(dw2) = dict.get("DW2") {
            let dw2 = dw2
                .as_array()
                .map_err(|_| PdfError::Font("Dw2 for CidFont is not an array".to_string()))?;
            let v = dw2
                .get(0)
                .ok_or(PdfError::Font("Dw2 elemnt error".to_string()))?
                .as_number()?
                .real();
            let w1 = dw2
                .get(1)
                .ok_or(PdfError::Font("Dw2 element error".to_string()))?
                .as_number()?
                .real();
            font.dw2 = Some((v, w1));
        }
        if let Some(w) = dict.get("W") {
            let wa = xref.read_object(w)?.as_array()?.to_owned();
            let widths = load_widths(&wa, xref)?;
            font.w = Some(widths);
        }

        if let Some(w2) = dict.get("W2") {
            let wa = xref.read_object(w2)?.as_array()?.to_owned();
            let w2v = load_widths_vertical(&wa, xref)?;
            font.w2 = Some(w2v);
        }
        if let Some(descriptor) = dict.get("FontDescriptor") {
            let descriptor_dict = xref.read_object(descriptor)?.as_dict()?.to_owned();
            let descriptor = Descriptor::try_new(descriptor_dict, xref)?;
            font.descriptor = Some(descriptor);
        }
        // TODO load font program

        Ok(font)
    }

    pub fn vertical_metrics(&self, code: &u32) -> Option<(f32, f32, f32)> {
        match &self.w2 {
            Some(w) => w.get(code).map(|x| x.to_owned()),
            None => match self.dw2 {
                Some((vy, h)) => {
                    let w0 = self.char_width(code).unwrap_or(0.0);
                    Some((h, w0 / 2.0, vy))
                }
                None => None,
            },
        }
    }

    pub fn char_width(&self, code: &u32) -> Result<f32> {
        if let Some(wd) = &self.w {
            if let Some(w) = wd.get(code) {
                return Ok(w.to_owned());
            }
            if let Some(w) = self.dw {
                return Ok(w as f32);
            }
            return Err(PdfError::Font(format!(
                "Type0 font char width is None: {:?}",
                code
            )));
        } else {
            // TODO
            if let Some(w) = self.dw {
                return Ok(w as f32);
            } else {
                return Err(PdfError::Font(format!(
                    "Type0 font char width is None: {:?}",
                    code
                )));
            }
        }
    }

    pub fn text_widths(&self, chars: &[CharCode]) -> Result<f32> {
        let mut total_widths = 0.0;
        if let Some(wd) = &self.w {
            for c in chars {
                total_widths += wd.get(&c.code).unwrap()
            }
        }
        Ok(total_widths)
    }
    pub fn fontfile(&self) -> Option<&[u8]> {
        match &self.descriptor {
            Some(desc) => desc.fontfile(),
            None => None,
        }
    }
}
fn load_widths_vertical(w: &PdfArray, _xref: &Xref) -> Result<HashMap<u32, (f32, f32, f32)>> {
    let mut res = HashMap::new();
    let n = w.len();
    let mut i = 0;
    while i < n {
        let obj1 = w.get(i).unwrap().as_number()?.integer() as u32;
        let obj2 = w.get(i + 1).unwrap();
        match obj2 {
            PdfObject::Array(arr) => {
                let wn = arr.len();
                let mut k = 0;
                let mut k = 0;
                let mut key = obj1;
                while k < wn {
                    let w1 = arr.get(k).unwrap().as_number()?.real();
                    let vx = arr.get(k + 1).unwrap().as_number()?.real();
                    let vy = arr.get(k + 2).unwrap().as_number()?.real();
                    k += 3;
                    res.insert(key, (w1, vx, vy));
                    key += 1;
                }
                i += 2;
            }
            PdfObject::Number(end) => {
                let start = obj1;
                let end = end.integer() as u32;
                let w1 = w.get(i + 2).unwrap().as_number()?.real();
                let vx = w.get(i + 3).unwrap().as_number()?.real();
                let vy = w.get(i + 4).unwrap().as_number()?.real();
                for key in start..=end {
                    res.insert(key, (w1, vx, vy));
                }
                i += 5;
            }
            _ => return Err(PdfError::Font("Dw2 format error".to_string())),
        }
    }

    Ok(res)
}

fn load_widths(w: &PdfArray, _xref: &Xref) -> Result<HashMap<u32, f32>> {
    let mut widths = HashMap::new();
    let n = w.len();
    let mut i = 0;
    while i < n {
        let obj1 = w.get(i).unwrap().as_number()?.integer() as u32;
        let obj2 = w.get(i + 1).unwrap();
        match obj2 {
            PdfObject::Array(arr) => {
                let mut start = obj1;
                for a in arr.iter() {
                    let aw = a.as_number()?.real();
                    widths.insert(start, aw);
                    start += 1;
                }
                i += 2;
            }
            PdfObject::Number(vn) => {
                let aw = w.get(i + 2).unwrap().as_number()?.real();
                let end = vn.integer() as u32;
                for k in obj1..end {
                    widths.insert(k, aw);
                }
                i += 3;
            }
            _ => {
                return Err(PdfError::Font(format!(
                    "CidFont w need PdfNumber or PdfArray got:{:?}",
                    obj2
                )));
            }
        }
    }
    Ok(widths)
}
