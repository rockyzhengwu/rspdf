use std::collections::HashMap;
use std::io::{Read, Seek};
use std::u32;

use freetype::GlyphSlot;

use crate::document::Document;
use crate::errors::{PDFError, PDFResult};
use crate::font::cmap::charcode::CharCode;
use crate::font::cmap::predefined::get_predefine_cmap;
use crate::font::cmap::CMap;
use crate::font::font_descriptor::FontDescriptor;
use crate::font::ft_font::FTFont;
use crate::object::{PDFArray, PDFObject};

#[derive(Debug, Clone)]
pub enum CIDCoding {
    GB,
    BIG5,
    JIS,
    KOREA,
    UCS2,
    CID,
    UTF16,
}

#[derive(Debug, Clone)]
pub enum CompositeFontType {
    Type1,
    TrueType,
}

#[derive(Debug, Clone)]
pub struct CompositeFont {
    basename: String,
    font_type: CompositeFontType,
    encoding: CMap,
    desc: FontDescriptor,
    ft_font: FTFont,
    cid_coding: Option<CIDCoding>,
    to_unicode: CMap,
    widths: HashMap<u32, f64>,
    dw: f64,
    widths_y: HashMap<u32, f64>,
    widths_v: HashMap<u32, (f64, f64)>,
    dwy: f64,
    dv: f64,
}
impl CompositeFont {
    pub fn basename(&self) -> &str {
        &self.basename
    }

    pub fn glyph_index_from_charcode(&self, charcode: &CharCode) -> Option<u32> {
        // just use cid as gid
        self.encoding.charcode_to_cid(charcode)
    }

    pub fn get_glyph(&self, gid: u32, scale: u32) -> Option<GlyphSlot> {
        self.ft_font.get_glyph(gid, scale)
    }

    pub fn decode_to_unicode(&self, bytes: &[u8]) -> Vec<String> {
        self.to_unicode.charcodes_to_unicode(bytes)
    }

    pub fn decode_chars(&self, bytes: &[u8]) -> Vec<CharCode> {
        let mut res = Vec::new();
        let mut offset: usize = 0;
        while offset < bytes.len() {
            if let Some(ch) = self.encoding.next_char(bytes, offset) {
                offset += ch.length() as usize;
                res.push(ch)
            }
        }
        res
    }

    pub fn is_vertical(&self) -> bool {
        self.encoding.wmode().map_or(false, |x| x == 1)
    }

    pub fn get_char_width(&self, charcode: &CharCode) -> f64 {
        if self.is_vertical() {
            if let Some(cid) = self.encoding.charcode_to_cid(charcode) {
                if let Some(w) = self.widths_y.get(&cid) {
                    return w.to_owned();
                }
            }
            self.dwy
        } else {
            if let Some(cid) = self.encoding.charcode_to_cid(charcode) {
                if let Some(w) = self.widths.get(&cid) {
                    return w.to_owned();
                }
            }
            self.dw
        }
    }
}

impl Default for CompositeFont {
    fn default() -> Self {
        CompositeFont {
            basename: String::new(),
            font_type: CompositeFontType::Type1,
            encoding: CMap::default(),
            desc: FontDescriptor::default(),
            ft_font: FTFont::default(),
            cid_coding: None,
            to_unicode: CMap::default(),
            widths: HashMap::new(),
            dw: 1000.0,
            widths_y: HashMap::new(),
            widths_v: HashMap::new(),
            dwy: -1000.0,
            dv: 880.0,
        }
    }
}

fn load_widths(w: &PDFArray) -> HashMap<u32, f64> {
    let mut widths = HashMap::new();
    let n = w.len();
    let mut i = 0;
    while i < n {
        let obj1 = w.get(i).unwrap().as_u32().unwrap();
        let obj2 = w.get(i + 1).unwrap();
        match obj2 {
            PDFObject::Arrray(arr) => {
                let mut start = obj1;
                for a in arr {
                    let aw = a.as_f64().unwrap();
                    widths.insert(start, aw);
                    start += 1;
                }
                i += 2;
            }
            PDFObject::Number(n) => {
                let aw = w.get(i + 2).unwrap().as_f64().unwrap();
                let end = n.as_i64() as u32;
                for k in obj1..end {
                    widths.insert(k, aw);
                }
                i += 3;
            }
            _ => {
                panic!("obj2 is need array or number")
            }
        }
    }
    widths
}

fn load_widths_vertical(w: &PDFArray, font: &mut CompositeFont) {
    let mut vs = HashMap::new();
    let mut widths = HashMap::new();
    let n = w.len();
    let mut i = 0;
    while i < n {
        let cid = w.get(i).unwrap().as_u32().unwrap();
        let obj2 = w.get(i + 1).unwrap();
        match obj2 {
            PDFObject::Arrray(arr) => {
                let w1 = arr.first().unwrap().as_f64().unwrap();
                let v0 = arr.get(1).unwrap().as_f64().unwrap();
                let v1 = arr.get(2).unwrap().as_f64().unwrap();
                widths.insert(cid, w1);
                vs.insert(cid, (v0, v1));
                i += 2;
            }
            PDFObject::Number(_) => {
                let end = obj2.as_u32().unwrap();
                let w1 = w.get(i + 2).unwrap().as_f64().unwrap();
                let v0 = w.get(i + 3).unwrap().as_f64().unwrap();
                let v1 = w.get(i + 4).unwrap().as_f64().unwrap();
                for c in cid..=end {
                    widths.insert(c, w1);
                    vs.insert(c, (v0, v1));
                }
                i += 5;
            }
            _ => {}
        }
    }
    font.widths_y = widths;
    font.widths_v = vs;

    //pass
}

pub fn load_encoding<T: Seek + Read>(
    font: &mut CompositeFont,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<()> {
    let enc = obj.get_value("Encoding");

    match enc {
        Some(o) => match o {
            PDFObject::Name(name) => {
                font.encoding = get_predefine_cmap(name.name());
                Ok(())
            }
            PDFObject::Indirect(_) => {
                let encs = doc.get_object_without_indriect(o)?;
                font.encoding = CMap::new_from_bytes(encs.bytes()?.as_slice())?;
                Ok(())
            }
            _ => Err(PDFError::FontFailure(format!(
                "Type0 Encoding type error:{:?}",
                obj
            ))),
        },
        None => Err(PDFError::FontFailure(format!(
            "Type0 missing Encoding:{:?}",
            obj
        ))),
    }
}
fn cid_collection<T: Seek + Read>(obj: &PDFObject, doc: &Document<T>) -> PDFResult<String> {
    match obj.get_value("CIDSystemInfo") {
        Some(info) => {
            let cidinfo = doc.get_object_without_indriect(info)?;
            let reg = cidinfo.get_value_as_string("Registry").unwrap()?;
            let order = cidinfo.get_value_as_string("Ordering").unwrap()?;
            Ok(format!("{}-{}", reg, order))
        }
        None => Ok(String::from("Identity")),
    }
}
fn load_unicode<T: Seek + Read>(
    font: &mut CompositeFont,
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<()> {
    if let Some(tu) = obj.get_value("ToUnicode") {
        let tounicode = doc.get_object_without_indriect(tu)?;

        match tounicode {
            PDFObject::Stream(s) => {
                let cmp = CMap::new_from_bytes(s.bytes().as_slice())?;
                font.to_unicode = cmp;
            }
            PDFObject::Name(name) => {
                let cmap = get_predefine_cmap(name.name());
                font.to_unicode = cmap;
            }
            _ => {
                //
            }
        }
    } else {
        let collection = cid_collection(obj, doc)?;
        match collection.as_str() {
            "Adobe-CNS1" => {
                font.to_unicode = get_predefine_cmap("Adobe-CNS1-UCS2");
            }
            "Adobe-GB1" => {
                font.to_unicode = get_predefine_cmap("Adobe-GB1-UCS2");
            }
            "Adobe-Japan1" => {
                font.to_unicode = get_predefine_cmap("Adobe-Japan1-UCS2");
            }
            "Adobe-Korea1" => {
                font.to_unicode = get_predefine_cmap("Adobe-Korea1-UCS2");
            }
            _ => {}
        }
    }
    Ok(())
}

fn font_type(dfont: &PDFObject) -> PDFResult<CompositeFontType> {
    match dfont.get_value_as_string("Subtype") {
        Some(subtype) => {
            let subtype = subtype?;
            match subtype.as_str() {
                "CIDFontType2" => Ok(CompositeFontType::TrueType),
                "CIDFontType0" => Ok(CompositeFontType::Type1),
                _ => Err(PDFError::FontFailure(format!(
                    "unknow subytype in dfont:{:?}",
                    subtype
                ))),
            }
        }
        _ => Err(PDFError::FontFailure(format!(
            "didn't have subytype in dfont:{:?}",
            dfont
        ))),
    }
}

pub fn load_composite_font<T: Seek + Read>(
    obj: &PDFObject,
    doc: &Document<T>,
) -> PDFResult<CompositeFont> {
    let mut font = CompositeFont::default();

    if let Some(s) = obj.get_value_as_string("BaseFont") {
        let name = s?;
        if name.find('+') == Some(6) {
            font.basename = name.split_once('+').unwrap().1.to_string();
        } else {
            font.basename = name;
        }
    }

    let dfont: PDFObject = match obj.get_value("DescendantFonts") {
        Some(v) => {
            let vv = doc.get_object_without_indriect(v)?;
            let arrs = vv.as_array()?;
            arrs.first().unwrap().to_owned()
        }
        None => {
            return Err(PDFError::FontFailure(format!(
                "descendants need array,got{:?}",
                obj
            )))
        }
    };
    let dfont = doc.get_object_without_indriect(&dfont)?;
    if let Some(desc) = dfont.get_value("FontDescriptor") {
        let desc = doc.get_object_without_indriect(desc)?;
        font.desc = FontDescriptor::new_from_object(&desc)?;
    }
    if let Some(embeded) = font.desc.embeded() {
        let embeded = doc.get_object_without_indriect(embeded)?;
        let ft_font = FTFont::try_new(embeded.bytes()?)?;
        font.ft_font = ft_font;
    } else {
        println!("not embeded");
        // TODO
        // load builtin font
    }
    load_encoding(&mut font, obj, doc)?;

    font.font_type = font_type(&dfont)?;
    // let collection = cid_collection(&dfont, doc)?;

    load_unicode(&mut font, obj, doc)?;
    if let Some(cidtogid) = dfont.get_value("CIDToGIDMap") {
        match &cidtogid {
            PDFObject::Stream(_) => {
                println!("CIDToGIDMap: {:?}", cidtogid);
            }
            PDFObject::Name(_) => {
                println!("cidgid:{:?}", cidtogid);
            }
            _ => {}
        }
    }
    if let Some(dw) = dfont.get_value("DW") {
        let dw = dw.as_f64()?;
        font.dw = dw;
    }
    if let Some(widtharray) = dfont.get_value("W") {
        let w_arr = doc.get_object_without_indriect(widtharray)?;
        font.widths = load_widths(w_arr.as_array()?);
    }
    if let Some(dw_v) = dfont.get_value("DW2") {
        let dw_v = dw_v.as_array()?;
        font.dwy = dw_v.first().unwrap().as_f64()?;
        font.dv = dw_v.last().unwrap().as_f64()?;
    }

    if let Some(widtharray) = dfont.get_value("W2") {
        load_widths_vertical(widtharray.as_array()?, &mut font);
    }

    Ok(font)
}

fn load_glyph(font: &mut CompositeFont, obj: &PDFObject) -> PDFResult<()> {
    unimplemented!()
    //
}
