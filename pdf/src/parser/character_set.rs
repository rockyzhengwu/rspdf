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
    ch.is_ascii_digit() || (b'a'..=b'f').contains(&ch) || (b'A'..=b'F').contains(&ch)
}

pub fn hex_to_u8(ch: &u8) -> u8 {
    match ch {
        b'0'..=b'9' => ch - b'0',
        b'a'..=b'f' => ch - b'a' + 10,
        b'A'..=b'F' => ch - b'A' + 10,
        _ => panic!("{} not a hex", ch),
    }
}

pub fn buf_to_number(buf: &[u8]) -> i64 {
    let mut res: i64 = 0;
    for c in buf {
        res = res * 10 + (c - b'0') as i64;
    }
    res
}

fn is_digit(ch: &u8) -> bool {
    ch.is_ascii_digit()
}

pub fn buf_to_real(buf: &[u8]) -> f64 {
    if buf.is_empty() {
        return 0_f64;
    }
    let mut i = 0;
    let flag: f64 = match buf[0] {
        43 => {
            i += 1;
            1_f64
        }
        45 => {
            i += 1;
            -1_f64
        }
        _ => 1_f64,
    };

    let mut ipart = 0_f64;
    while i < buf.len() && is_digit(&buf[i]) {
        ipart = ipart * 10_f64 + (buf[i] - b'0') as f64;
        i += 1
    }
    if i < buf.len() && buf[i] != b'.' {
        return flag * ipart;
    } else if i < buf.len() && buf[i] == b'.' {
        i += 1;
        let mut dpart = 0_f64;
        let mut n = 1_f64;
        while i < buf.len() && is_digit(&buf[i]) {
            n *= 10_f64;
            dpart = dpart * 10_f64 + (buf[i] - b'0') as f64;
            i += 1
        }
        return flag * (ipart + dpart / n);
    }

    flag * ipart
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitesapce() {
        assert!(is_whitespace(32));
    }
}
