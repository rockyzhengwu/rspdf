use crate::error::{PdfError, Result};
use crate::object::dictionary::PdfDict;
use flate2::bufread::ZlibDecoder;
use std::io::Read;

pub fn flate_decode(input: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(input);
    let mut buf: [u8; 1] = [0; 1];
    let mut decompressed: Vec<u8> = Vec::new();
    loop {
        match decoder.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                decompressed.push(buf[0]);
            }
            Err(e) => {
                //return Err(PdfError::Filter(format!("FlateDecode error :{:?}", e)));
                println!("Flated Decode Error:{:?}", e);
                break;
            }
        }
    }
    if let Some(params) = params {
        if let Some(predictor) = params.get("Predictor") {
            let columns = match params.get("Columns") {
                Some(v) => v.integer()? as usize,
                None => 1,
            };

            let colors = match params.get("Colors") {
                Some(v) => v.integer()?,
                None => 1,
            };

            let bits_per_component = match params.get("BitsPerComponent") {
                Some(v) => v.integer()?,
                None => 8,
            };

            let bits_per_pixel = colors as usize * bits_per_component as usize;
            let bytes_per_pixel = (bits_per_pixel + 7) / 8;
            let row_size = (columns * bits_per_pixel + 7) / 8;
            let mut decoded_data =
                Vec::with_capacity(decompressed.len() - decompressed.len() / (row_size + 1));
            let p = predictor.integer().map_err(|_| {
                PdfError::Filter("Flateddecoder Predictor is not a number".to_string())
            })?;
            if p >= 10 {
                let mut i = 0;
                let mut prev_row = vec![0u8; row_size];
                while i < decompressed.len() {
                    let filter_type = decompressed[i];
                    i += 1;
                    let row_data = &decompressed[i..i + row_size];
                    let decoded_row = match filter_type {
                        0 => row_data.to_vec(),
                        1 => png_sub(row_data, bytes_per_pixel),
                        2 => png_up(row_data, &prev_row),
                        3 => png_average(row_data, &prev_row, bytes_per_pixel),
                        4 => png_paeth(row_data, &prev_row, bytes_per_pixel),
                        _ => {
                            return Err(PdfError::Filter(format!(
                                "Unknown PNG predictor {}",
                                filter_type
                            )));
                        }
                    };
                    decoded_data.extend_from_slice(&decoded_row);
                    prev_row.clone_from_slice(&decoded_row);
                    i += row_size;
                    // optimum
                }
                return Ok(decoded_data);
            }
        }
    }

    Ok(decompressed)
}

fn png_sub(row: &[u8], bytes_per_pixel: usize) -> Vec<u8> {
    let mut result = row.to_vec();
    for i in bytes_per_pixel..row.len() {
        result[i] = result[i].wrapping_add(result[i - bytes_per_pixel]);
    }
    result
}

fn png_up(row: &[u8], prev_row: &[u8]) -> Vec<u8> {
    row.iter()
        .zip(prev_row.iter())
        .map(|(&r, &p)| r.wrapping_add(p))
        .collect()
}

fn png_average(row: &[u8], prev_row: &[u8], bytes_per_pixel: usize) -> Vec<u8> {
    let mut result = row.to_vec();
    for i in 0..row.len() {
        let left = if i >= bytes_per_pixel {
            result[i - bytes_per_pixel]
        } else {
            0
        };
        let up = prev_row[i];
        result[i] = row[i].wrapping_add(((left as u16 + up as u16) / 2) as u8);
    }
    result
}

/// PNG Paeth filter: Uses Paeth predictor function.
fn png_paeth(row: &[u8], prev_row: &[u8], bytes_per_pixel: usize) -> Vec<u8> {
    let mut result = row.to_vec();
    for i in 0..row.len() {
        let left = if i >= bytes_per_pixel {
            result[i - bytes_per_pixel]
        } else {
            0
        };
        let up = prev_row[i];
        let up_left = if i >= bytes_per_pixel {
            prev_row[i - bytes_per_pixel]
        } else {
            0
        };
        result[i] = row[i].wrapping_add(paeth_predictor(left, up, up_left));
    }
    result
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = (p - a as i16).abs();
    let pb = (p - b as i16).abs();
    let pc = (p - c as i16).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::flate_decode;

    #[test]
    fn test_flated_decode() {
        let encoded = [
            0x78, 0x9c, 0x4b, 0xcb, 0xcf, 0x07, 0x00, 0x02, 0x82, 0x01, 0x45,
        ];
        let res = flate_decode(&encoded, None).unwrap();
        assert_eq!(res, b"foo");
    }
}
