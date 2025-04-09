use crate::error::Result;
use crate::object::dictionary::PdfDict;
use crate::xref::Xref;

#[derive(Debug, Default, Clone)]
pub struct Descriptor {
    dict: PdfDict,
    flags: Option<u8>,
    italic_angle: Option<f32>,
    ascent: f32,
    descent: f32,
    leading: f32,
    cap_height: f32,
    x_height: f32,
    stem_v: f32,
    stem_h: f32,
    avg_width: f32,
    missing_width: f32,
    max_width: f32,
    fontfile: Option<Vec<u8>>,
    is_embed: bool,
}

impl Descriptor {
    pub fn try_new(dict: PdfDict, xref: &Xref) -> Result<Self> {
        let mut desc = Descriptor {
            dict,
            flags: None,
            is_embed: false,
            ..Default::default()
        };
        if let Some(f) = desc.dict.get("Flags") {
            let v = f.integer()?;
            desc.flags = Some(v as u8);
        }
        if let Some(i) = desc.dict.get("ItalicAngle") {
            let v = i.as_number()?.real();
            desc.italic_angle = Some(v);
        }
        if let Some(a) = desc.dict.get("Ascent") {
            let v = a.as_number()?.real();
            desc.ascent = v;
        }
        if let Some(d) = desc.dict.get("Descent") {
            let v = d.as_number()?.real();
            desc.descent = v;
        }
        if let Some(l) = desc.dict.get("Leading") {
            let l = l.as_number()?.real();
            desc.leading = l;
        }
        if let Some(c) = desc.dict.get("CapHeight") {
            let ch = c.as_number()?.real();
            desc.cap_height = ch;
        }
        if let Some(x) = desc.dict.get("XHeight") {
            let x = x.as_number()?.real();
            desc.x_height = x;
        }
        if let Some(sv) = desc.dict.get("StemV") {
            let s = sv.as_number()?.real();
            desc.stem_v = s;
        }
        if let Some(sh) = desc.dict.get("StemH") {
            let s = sh.as_number()?.real();
            desc.stem_h = s;
        }
        if let Some(av) = desc.dict.get("AvgWidth") {
            let a = av.as_number()?.real();
            desc.avg_width = a;
        }
        if let Some(ms) = desc.dict.get("MissingWidth") {
            let m = ms.as_number()?.real();
            desc.missing_width = m;
        }
        if let Some(ms) = desc.dict.get("MaxWidth") {
            let m = ms.as_number()?.real();
            desc.max_width = m;
        }
        let f1 = desc.dict.get("FontFile");
        let f2 = desc.dict.get("FontFile2");
        let f3 = desc.dict.get("FontFile3");
        let ff = f1.or(f2).or(f3);
        if let Some(fo) = ff {
            let o = xref.read_object(fo)?.to_stream()?;
            desc.fontfile = Some(o.decode_data(Some(xref))?);
            desc.is_embed = true;
        }

        Ok(desc)
    }

    pub fn fontfile(&self) -> Option<&[u8]> {
        if let Some(v) = &self.fontfile {
            Some(v.as_slice())
        } else {
            None
        }
    }

    pub fn flags(&self) -> Option<&u8> {
        self.flags.as_ref()
    }
    pub fn is_symbolic(&self) -> bool {
        if let Some(flag) = self.flags {
            (flag & 4) == 0
        } else {
            false
        }
    }

    pub fn is_embed(&self) -> bool {
        self.is_embed
    }
}
