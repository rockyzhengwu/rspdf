use crate::errors::{PDFError, PDFResult};
use crate::filter::Filter;
use crate::lexer::is_white;

#[derive(Default)]
pub struct ASCIIHexDecode {}

fn hex_to_decimal(c: &u8) -> PDFResult<i32> {
    match c {
        b'0'..=b'9' => Ok((c - b'0') as i32),
        b'a'..=b'z' => Ok((c - b'a') as i32 + 10),
        b'A'..=b'Z' => Ok((c - b'A') as i32 + 10),
        _ => Err(PDFError::Filter(format!(
            "Got unexpected character in hex decode:{:?}",
            c
        ))),
    }
}

impl Filter for ASCIIHexDecode {
    fn decode(
        &self,
        buf: &[u8],
        _param: Option<crate::object::PDFDictionary>,
    ) -> crate::errors::PDFResult<Vec<u8>> {
        let mut first = 0;
        let mut f = false;
        let mut result: Vec<u8> = Vec::new();
        for c in buf {
            if is_white(c) {
                continue;
            }
            if *c == b'>' {
                break;
            }
            let n = hex_to_decimal(c)?;
            if !f {
                first = n;
                f = true;
            } else {
                result.push((((first << 4) + n) & 0xff) as u8);
                f = false;
            }
        }
        if !f {
            result.push((first << 4 & 0xff) as u8)
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::asciihex_decode::ASCIIHexDecode;
    use crate::filter::Filter;

    #[test]
    fn test_decode() {
        let decoder = ASCIIHexDecode::default();
        let hex = "68656c6c6f20776f726c64g>";
        let result = decoder.decode(hex.as_bytes(), None).unwrap();
        let expected = "hello world";
        assert_eq!(expected, String::from_utf8(result).unwrap());
    }
}
