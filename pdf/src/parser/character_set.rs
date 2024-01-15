use std::usize;

const PDF_CHARACTER_SET: [char; 256] = [
    'W', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'W', 'W', 'R', 'W', 'W', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'W', 'R', 'R', 'R', 'R', 'D',
    'R', 'R', 'D', 'D', 'R', 'N', 'R', 'N', 'N', 'D', 'N', 'N', 'N', 'N', 'N', 'N', 'N', 'N', 'N',
    'N', 'R', 'R', 'D', 'R', 'D', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'D', 'R', 'D', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'D', 'R', 'D', 'R', 'R', 'W', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R',
    'R', 'R', 'R', 'R', 'R', 'R', 'R', 'R', 'W',
];

pub fn is_whitespace(ch: u8) -> bool {
    PDF_CHARACTER_SET[ch as usize] == 'W'
}

pub fn is_number(ch: u8) -> bool {
    PDF_CHARACTER_SET[ch as usize] == 'N'
}

pub fn is_delimiter(ch: u8) -> bool {
    PDF_CHARACTER_SET[ch as usize] == 'D'
}

pub fn is_end_of_line(ch: u8) -> bool {
    matches!(ch, b'\r' | b'\n')
}

pub fn is_regular(ch: u8) -> bool {
    PDF_CHARACTER_SET[ch as usize] == 'R'
}
pub fn is_xdigit(ch: u8) -> bool {
    (ch < 10) || (b'a'..=b'f').contains(&ch) || (b'A'..=b'F').contains(&ch)
}

pub fn hex_to_u8(ch: u8) -> u8 {
    match ch {
        0..=9 => ch,
        b'a'..=b'f' => ch - b'a' + 10,
        b'A'..=b'F' => ch - b'A' + 10,
        _ => panic!("{} not a hex", ch),
    }
}

pub fn is_octal_digit(ch: u8) -> bool {
    ch <= 7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitesapce() {
        assert!(is_whitespace(32));
    }
}
