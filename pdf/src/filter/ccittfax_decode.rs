use crate::errors::{PDFError, PDFResult};
use crate::object::PDFObject;
use fax::{
    decoder::{decode_g4, pels},
    Color,
};

pub struct BitReader {
    data: Vec<u8>,
    current_index: usize,
}

impl BitReader {
    pub fn new(data: Vec<u8>) -> Self {
        BitReader {
            data,
            current_index: 0,
        }
    }

    pub fn peek_u16(&self, bits: usize) -> PDFResult<u16> {
        if bits > 16 {
            return Err(PDFError::Filter(
                "ccitdecoder reader only peek 16 bits once".to_string(),
            ));
        }

        if (self.current_index + bits) > self.data.len() * 8 {
            return Err(PDFError::Filter("ccitdecoder reader eof".to_string()));
        }
        let n = self.current_index / 8;
        let delta = self.current_index % 8;
        let current_value = self.data.get(n).unwrap().to_owned() as u16;
        let current_remain = 8 - delta;

        if bits < current_remain {
            let v = current_value & ((1 << current_remain) - 1);
            Ok(v >> (current_remain - bits))
        } else {
            let first = current_value & ((1 << current_remain) - 1);
            let mut result = first << (bits - current_remain);
            let mut remain = bits - current_remain;
            let mut nn = n + 1;

            while remain > 0 {
                let nv = self.data.get(nn).unwrap().to_owned() as u16;
                if remain >= 8 {
                    result += nv << (remain - 8);
                    remain -= 8;
                } else {
                    result += nv >> (remain);
                    break;
                }
                nn += 1;
            }
            Ok(result)
        }
    }

    pub fn move_bits(&mut self, bits: usize) {
        self.current_index += bits;
    }
}

struct FaxDecoder {
    reader: BitReader,
}

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
    // TODO fix this just use fax right now
    let mut res = Vec::with_capacity((columns * rows) as usize);
    decode_g4(
        buf.iter().cloned(),
        columns as u16,
        Some(rows as u16),
        |line| {
            res.extend(pels(line, columns as u16).map(|c| match c {
                Color::Black => 0,
                Color::White => 255,
            }));
        },
    );

    Ok(res)
}
