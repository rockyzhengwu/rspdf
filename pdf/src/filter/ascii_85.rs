use crate::character;
use crate::error::{PdfError, Result};

pub fn ascii_85_decode(input: &[u8]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut buffer = Vec::new();
    let mut input = input.iter().peekable();
    while let Some(c) = input.next() {
        match c {
            b'~' => {
                if input.next() == Some(&b'>') {
                    break;
                } else {
                    return Err(PdfError::Filter(
                        "Invalid end sequence in ASCII85 stream".to_string(),
                    ));
                }
            }
            b'z' => {
                if !buffer.is_empty() {
                    return Err(PdfError::Filter(
                        "`z` cannot appear inside a group".to_string(),
                    ));
                }
                result.extend_from_slice(&[0, 0, 0, 0]);
            }
            &character::SPACE
            | &character::NULL
            | &character::CARRIAGE_RETURN
            | &character::FORM_FEED
            | &character::LINE_FEED
            | &character::HORIZONTAL_TAB => continue,
            b'!'..=b'u' => buffer.push((c.to_owned() as u32) - 33),
            _ => {
                return Err(PdfError::Filter(format!(
                    "Invalid character in ASCII85 stream: '{}'",
                    c
                )))
            }
        }
        if buffer.len() == 5 {
            let value = buffer.iter().fold(0_u32, |acc, &b| acc * 85 + b);
            result.extend_from_slice(&value.to_be_bytes());
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        let mut padded = buffer.clone();
        while padded.len() < 5 {
            padded.push(84);
        }

        let value = padded.iter().fold(0_u32, |acc, &b| acc * 85 + b);
        let remaining_bytes = buffer.len() - 1;
        result.extend_from_slice(&value.to_be_bytes()[..remaining_bytes]);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::ascii_85_decode;

    #[test]
    fn test_ascii_85_decode() {
        let input = r#"87cURD]j7BEbo80~>"#;
        let result = ascii_85_decode(input.as_bytes()).unwrap();
        assert_eq!(result, b"Hello world!");

        let content = r#"9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c~>"#;
        let result = ascii_85_decode(content.as_bytes()).unwrap();
        let expected = r#"Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure."#;
        assert_eq!(result, expected.as_bytes())
    }
}
