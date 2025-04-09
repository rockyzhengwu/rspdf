use crate::character;
use crate::error::{PdfError, Result};

pub fn ascii_hex_decode(input: &[u8]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut hex_buffer = String::new();

    for c in input {
        match c {
            &b'>' => break,
            &character::SPACE
            | &character::NULL
            | &character::CARRIAGE_RETURN
            | &character::FORM_FEED
            | &character::LINE_FEED
            | &character::HORIZONTAL_TAB => continue,
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => hex_buffer.push(c.to_owned() as char),
            _ => {
                return Err(PdfError::Filter(format!(
                    "Invalid character in ASCIIHexDecode stream: '{}'",
                    c
                )))
            }
        }
        if hex_buffer.len() == 2 {
            let byte = u8::from_str_radix(&hex_buffer, 16).map_err(|_| {
                PdfError::Filter(format!("Failed to parse hex pair: {}", hex_buffer))
            })?;
            bytes.push(byte);
            hex_buffer.clear();
        }
    }
    if !hex_buffer.is_empty() {
        let padded_hex = format!("{}0", hex_buffer);
        let byte = u8::from_str_radix(&padded_hex, 16)
            .map_err(|_| PdfError::Filter(format!("Failed to parse padded hex: {}", padded_hex)))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {

    use super::ascii_hex_decode;

    #[test]
    fn test_ascii_hex_decode() {
        let raw = r#"48656c6c6f20776f726c64>"#;
        let result = ascii_hex_decode(raw.as_bytes()).unwrap();
        assert_eq!(result, b"Hello world");
    }
}
