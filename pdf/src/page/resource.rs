use crate::error::{PdfError, Result};
use crate::object::array::PdfArray;
use crate::object::dictionary::PdfDict;
use crate::object::PdfObject;
use crate::xref::Xref;

#[derive(Debug, Clone)]
pub struct Resources {
    ext_g_state: Option<PdfDict>,
    color_space: Option<PdfDict>,
    pattern: Option<PdfDict>,
    shading: Option<PdfDict>,
    x_object: Option<PdfDict>,
    font: Option<PdfDict>,
    proc_set: Option<PdfArray>,
    properties: Option<PdfDict>,
}

impl Resources {
    pub fn try_new(resource: &PdfDict, xref: &Xref) -> Result<Self> {
        let mut resources = Resources {
            ext_g_state: None,
            color_space: None,
            pattern: None,
            shading: None,
            x_object: None,
            font: None,
            proc_set: None,
            properties: None,
        };
        if let Some(ext) = resource.get("ExtGState") {
            resources.ext_g_state = Some(xref.read_object(ext)?.as_dict()?.to_owned());
        }
        if let Some(cl) = resource.get("ColorSpace") {
            resources.color_space = Some(xref.read_object(cl)?.as_dict()?.to_owned());
        }
        if let Some(p) = resource.get("Pattern") {
            resources.pattern = Some(xref.read_object(p)?.as_dict()?.to_owned());
        }
        if let Some(s) = resource.get("Shading") {
            resources.pattern = Some(xref.read_object(s)?.as_dict()?.to_owned());
        }
        if let Some(x) = resource.get("XObject") {
            resources.x_object = Some(xref.read_object(x)?.as_dict()?.to_owned());
        }
        if let Some(font) = resource.get("Font") {
            resources.font = Some(xref.read_object(font)?.as_dict()?.to_owned());
        }

        if let Some(proc) = resource.get("ProcSet") {
            resources.proc_set = Some(xref.read_object(proc)?.as_array()?.to_owned());
        }

        if let Some(pro) = resource.get("Properties") {
            resources.properties = Some(xref.read_object(pro)?.as_dict()?.to_owned());
        }
        Ok(resources)
    }

    pub fn lookup_ext_g_state(&self, name: &str, xref: &Xref) -> Result<PdfDict> {
        match &self.ext_g_state {
            Some(ext) => {
                if let Some(d) = ext.get(name) {
                    let res = xref.read_object(d)?.as_dict()?.to_owned();
                    Ok(res)
                } else {
                    Err(PdfError::Page(format!(
                        "Page Resource not found:{:?}",
                        name
                    )))
                }
            }
            None => Err(PdfError::Page(
                "Page Resource ExtGState is None".to_string(),
            )),
        }
    }

    pub fn lookup_font(&self, name: &str, xref: &Xref) -> Result<PdfDict> {
        match &self.font {
            Some(fd) => {
                if let Some(f) = fd.get(name) {
                    let font = xref.read_object(f)?.as_dict()?.to_owned();
                    Ok(font)
                } else {
                    Err(PdfError::Page(format!(
                        "Font Resource not found {:?}",
                        name
                    )))
                }
            }
            None => Err(PdfError::Page("Page Resource Font is None".to_string())),
        }
    }

    pub fn fonts(&self) -> Option<&PdfDict> {
        self.font.as_ref()
    }

    pub fn lookup_color(&self, name: &str) -> Option<&PdfObject> {
        if let Some(d) = &self.color_space {
            d.get(name)
        } else {
            None
        }
    }

    pub fn lookup_pattern(&self, pname: &str) -> Option<&PdfObject> {
        if let Some(p) = &self.pattern {
            p.get(pname)
        } else {
            None
        }
    }

    pub fn lookup_xobject(&self, name: &str) -> Option<&PdfObject> {
        match &self.x_object {
            Some(x) => x.get(name),
            None => None,
        }
    }
}
