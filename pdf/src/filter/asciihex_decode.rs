use crate::errors::{PDFError, PDFResult};
use crate::parser::character_set::is_whitespace;

fn hex_to_decimal(c: &u8) -> PDFResult<i32> {
    match c {
        b'0'..=b'9' => Ok((c - b'0') as i32),
        b'a'..=b'f' => Ok((c - b'a') as i32 + 10),
        b'A'..=b'F' => Ok((c - b'A') as i32 + 10),
        _ => Err(PDFError::Filter(format!(
            "Got unexpected character in hex decode:{:?}",
            c
        ))),
    }
}

pub fn asciihex_decode(buf: &[u8]) -> PDFResult<Vec<u8>> {
    let mut first = 0;
    let mut f = false;
    let mut result: Vec<u8> = Vec::new();
    for c in buf {
        if is_whitespace(*c) {
            continue;
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
    if f {
        result.push((first << 4 & 0xff) as u8)
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use crate::filter::asciihex_decode::asciihex_decode;

    #[test]
    fn test_decode() {
        let hex = "68656c6c6f20776f726c64";
        let result = asciihex_decode(hex.as_bytes()).unwrap();
        let expected = "hello world";
        assert_eq!(expected, String::from_utf8(result).unwrap());
        let hex = "28AA0C540E5306A010B511D92BB007D4279534D0047F0359118A03BA540454041B0C4B27";
        let result = asciihex_decode(hex.as_bytes()).unwrap();
        assert_eq!(result.len(), 36);
    }
}
