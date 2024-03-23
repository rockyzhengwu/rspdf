use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;

pub fn decode_point(obj: &PDFObject, name: &str) -> PDFResult<Option<[f32; 3]>> {
    if let Some(w) = obj.get_value(name) {
        let vs = w.as_array()?;
        if vs.len() != 3 {
            return Err(PDFError::ColorError(format!(
                "Decode color point {} need 3 parama:{:?}",
                name, w
            )));
        }
        let x = vs.first().unwrap().as_f32()?;
        let y = vs.get(1).unwrap().as_f32()?;
        let z = vs.last().unwrap().as_f32()?;
        return Ok(Some([x, y, z]));
    }
    Ok(None)
}
