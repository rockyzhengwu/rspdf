use crate::errors::PDFResult;
use crate::object::PDFObject;

pub fn fax_decode(buf: &[u8], param: Option<&PDFObject>) -> PDFResult<Vec<u8>> {
    let mut k = 0;
    let mut columns = 1728;
    let mut rows = 0;
    let mut end_of_block = true;
    let mut end_byte_begin = false;
    let mut blackis1 = false;
    let mut damaged_rows_before_error = 0;

    if let Some(p) = param {
        if let Some(v) = p.get_value("K") {
            k = v.as_i32()?;
        }
        if let Some(c) = p.get_value("Columns") {
            columns = c.as_u32()?;
        }
        if let Some(r) = p.get_value("Rows") {
            rows = r.as_u32()?;
        }
        if let Some(e) = p.get_value("EndOfBlock") {
            end_of_block = e.as_bool()?;
        }
        if let Some(e) = p.get_value("EncodedByteAlign") {
            end_byte_begin = e.as_bool()?;
        }
        if let Some(b) = p.get_value("BlackIs1") {
            blackis1 = b.as_bool()?;
        }
        if let Some(d) = p.get_value("DamagedRowsBeforeError") {
            damaged_rows_before_error = d.as_u32()?;
        }
    }

    println!("{:?},{:?},{:?}", k, columns, rows);

    unimplemented!()
}
