//use group4::decode_g4;

use crate::{
    error::{PdfError, Result},
    object::dictionary::PdfDict,
};
use fax::decoder::{decode_g4, pels};
use fax::Color;

mod bitreader;
mod fax_table;
mod group4;

pub struct Param {
    k: i8,
    columns: u16,
    rows: u16,
    end_of_line: bool,
    encoded_byte_align: bool,
    end_of_block: bool,
    black_is_1: bool,
    damaged_rows_before_error: u16,
}
impl Default for Param {
    fn default() -> Self {
        Param {
            k: 0,
            columns: 1728,
            rows: 0,
            end_of_line: false,
            end_of_block: true,
            encoded_byte_align: false,
            black_is_1: false,
            damaged_rows_before_error: 0,
        }
    }
}
impl Param {
    pub fn try_new(dict: &PdfDict) -> Result<Self> {
        let mut param = Param::default();
        if let Some(k) = dict.get("K") {
            param.k = k
                .as_number()
                .map_err(|_| {
                    PdfError::Filter("ccittfax_decode k param is not a number".to_string())
                })?
                .integer() as i8;
        }
        if let Some(cl) = dict.get("Columns") {
            param.columns = cl.integer().map_err(|_| {
                PdfError::Filter("ccittfax_decode Columns is not a number".to_string())
            })? as u16;
        }
        if let Some(rw) = dict.get("Rows") {
            param.rows = rw
                .integer()
                .map_err(|_| PdfError::Filter("ccittfax_decode Rows is not a number".to_string()))?
                as u16;
        }
        if let Some(black) = dict.get("BlackIs1") {
            param.black_is_1 = black.as_bool()?.0;
        }
        Ok(param)
    }
}

use std::io::Write;
pub fn ccittfax_decode(buf: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>> {
    let mut f = std::fs::File::create("ccittfax_decode").unwrap();
    f.write_all(buf).unwrap();

    let p = match params {
        Some(d) => Param::try_new(d)?,
        None => Param::default(),
    };
    let width = p.columns;
    let height = if p.rows > 0 { Some(p.rows) } else { None };
    let mut res = Vec::new();
    if p.k < 0 {
        //return decode_g4(buf, &p);
        decode_g4(buf.to_vec().into_iter(), width, height, |line| {
            res.extend(pels(line, p.columns).map(|c| match c {
                Color::White => 255,
                Color::Black => 0,
            }));
        });
        return Ok(res);
    }

    unimplemented!("ccittfax_decode group3 and mix goroup3 not implement");
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Read};

    use crate::object::{dictionary::PdfDict, number::PdfNumber, PdfObject};

    use super::ccittfax_decode;

    #[test]
    fn test_decode_g4() {
        let mut f = std::fs::File::open("./tests/resources/ccittfax_data").unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let mut pd = HashMap::new();
        pd.insert(
            "Columns".to_string(),
            PdfObject::Number(PdfNumber::Integer(2869)),
        );
        pd.insert("K".to_string(), PdfObject::Number(PdfNumber::Integer(-1)));
        let params = PdfDict::new(pd);
        let images = ccittfax_decode(&buffer, Some(&params)).unwrap();
        assert_eq!(images.len() / 2869, 600);
    }
}
