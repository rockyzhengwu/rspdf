use std::collections::HashMap;

use crate::error::{PdfError, Result};
use crate::object::dictionary::PdfDict;

pub fn lzw_decode(input: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>> {
    let mut dictionary: HashMap<u16, Vec<u8>> =
        (0..=255).map(|i| (i as u16, vec![i as u8])).collect();

    let mut result = Vec::new();
    let mut code_size = 9;
    let mut next_code: u16 = 258;
    let mut prev_entry: Option<Vec<u8>> = None;

    let mut current_byte = 0;
    let mut buffer: u16 = 0;
    let mut bits_in_buffer = 0;

    for &byte in input {
        current_byte = (current_byte << 8) | byte as u16;
        bits_in_buffer += 8;

        while bits_in_buffer >= code_size {
            let mask = (1 << code_size) - 1;
            buffer = (current_byte >> (bits_in_buffer - code_size)) & mask;
            bits_in_buffer -= code_size;

            if buffer == 256 {
                dictionary = (0..=255).map(|i| (i as u16, vec![i as u8])).collect();
                next_code = 257;
                code_size = 9;
                prev_entry = None;
                continue;
            } else if buffer == 257 {
                // End of data
                break;
            }

            let entry = if let Some(entry) = dictionary.get(&buffer) {
                entry.clone()
            } else if let Some(prev) = &prev_entry {
                let mut new_entry = prev.clone();
                new_entry.push(prev[0]);
                new_entry
            } else {
                return Err(PdfError::Filter(
                    "Invalid LZW stream: unknown code".to_string(),
                ));
            };

            result.extend_from_slice(&entry);

            if let Some(prev) = prev_entry {
                if next_code < (1 << code_size) {
                    let mut new_entry = prev.clone();
                    new_entry.push(entry[0]);
                    dictionary.insert(next_code, new_entry);
                    next_code += 1;

                    if next_code == (1 << code_size) && code_size < 12 {
                        code_size += 1;
                    }
                    if code_size > 12 {
                        return Err(PdfError::Filter(
                            "LzwFilter code max_size is 12".to_string(),
                        ));
                    }
                }
            }

            prev_entry = Some(entry);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::lzw_decode;

    #[test]
    fn test_lzw_decode() {
        let encoded = [0x80, 0x0B, 0x60, 0x50, 0x22, 0x0C, 0x0C, 0x85, 0x01];
        let res = lzw_decode(&encoded, None).unwrap();
        println!("{:?}", res);
        assert_eq!(res, [45, 45, 45, 45, 45, 65, 45, 45, 45, 66]);
    }
}
