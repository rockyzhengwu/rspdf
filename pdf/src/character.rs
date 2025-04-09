use crate::error::{PdfError, Result};

pub(crate) const NULL: u8 = 0;
pub(crate) const HORIZONTAL_TAB: u8 = 9;
pub(crate) const LINE_FEED: u8 = 10;
pub(crate) const FORM_FEED: u8 = 12;
pub(crate) const CARRIAGE_RETURN: u8 = 13;
pub(crate) const SPACE: u8 = 32;

pub(crate) const LEFT_PARENTHESIS: u8 = b'(';
pub(crate) const RIGHT_PARENTHESIS: u8 = b')';
pub(crate) const LESS_THAN_SIGN: u8 = b'<';
pub(crate) const GREATER_THAN_SIGN: u8 = b'>';
pub(crate) const LEFT_SQUARE_BRACKET: u8 = b'[';
pub(crate) const RIGHT_SQUARE_BRACKET: u8 = b']';
pub(crate) const LEFT_CURLY_BRACKET: u8 = b'{';
pub(crate) const RIGHT_CURLY_BRACKET: u8 = b'}';
pub(crate) const SOLIDUS: u8 = b'/';
pub(crate) const PERCENT_SIGN: u8 = b'%';
pub(crate) const NUMBER_SIGN: u8 = b'#';

pub(crate) const REVERSE_SOLIDUS: u8 = b'\\';

pub(crate) fn is_white_space(ch: &u8) -> bool {
    matches!(
        ch,
        &NULL | &HORIZONTAL_TAB | &LINE_FEED | &FORM_FEED | &CARRIAGE_RETURN | &SPACE
    )
}

pub(crate) fn is_delimiter(ch: &u8) -> bool {
    matches!(
        ch,
        &LEFT_PARENTHESIS
            | &RIGHT_PARENTHESIS
            | &LESS_THAN_SIGN
            | &GREATER_THAN_SIGN
            | &LEFT_SQUARE_BRACKET
            | &RIGHT_SQUARE_BRACKET
            | &LEFT_CURLY_BRACKET
            | &RIGHT_CURLY_BRACKET
            | &SOLIDUS
            | &PERCENT_SIGN
    )
}

pub(crate) fn is_number(ch: &u8) -> bool {
    matches!(ch, &b'.' | &b'+' | &b'-' | b'0'..=b'9')
}

pub fn u32_from_buffer(buf: &[u8]) -> Result<u32> {
    let mut n: u32 = 0;
    for c in buf {
        if !matches!(c, b'0'..=b'9') {
            return Err(PdfError::Character(
                "usize_from_buffer param need between 0-9".to_string(),
            ));
        }
        n = n * 10 + (c - 48) as u32;
    }
    Ok(n)
}

pub fn u16_from_buffer(buf: &[u8]) -> Result<u16> {
    let mut n: u16 = 0;
    for c in buf {
        if !matches!(c, b'0'..=b'9') {
            return Err(PdfError::Character(
                "usize_from_buffer param need between 0-9".to_string(),
            ));
        }
        n = n * 10 + (c - 48) as u16;
    }
    Ok(n)
}

pub fn usize_from_buffer(buf: &[u8]) -> Result<usize> {
    let mut n: usize = 0;
    for c in buf {
        if !matches!(c, b'0'..=b'9') {
            return Err(PdfError::Character(format!(
                "usize_from_buffer param need between 0-9 got:{:?}",
                c
            )));
        }
        n = n * 10 + (c - 48) as usize;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use crate::character::is_number;

    #[test]
    fn test_is_number() {
        assert!(is_number(&b'2'));
        assert!(is_number(&b'0'));
        assert!(is_number(&b'9'));
        assert!(is_number(&b'-'));
        assert!(is_number(&b'+'));
    }
}
