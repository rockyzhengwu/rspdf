#[derive(Debug, PartialEq, Clone)]
pub enum PdfNumber {
    Integer(i32),
    Real(f32),
}

impl PdfNumber {
    pub fn from_buffer(buffer: &[u8], is_real: bool) -> Self {
        if is_real {
            let r = real_from_buffer(buffer);
            PdfNumber::Real(r)
        } else {
            let i = integer_from_buffer(buffer);
            PdfNumber::Integer(i)
        }
    }
    pub fn integer(&self) -> i32 {
        match self {
            PdfNumber::Integer(v) => v.to_owned(),
            PdfNumber::Real(r) => r.to_owned() as i32,
        }
    }

    pub fn real(&self) -> f32 {
        match self {
            PdfNumber::Integer(v) => v.to_owned() as f32,
            PdfNumber::Real(r) => r.to_owned(),
        }
    }
}

pub fn integer_from_buffer(buf: &[u8]) -> i32 {
    let mut res: i32 = 0;
    let mut i: usize = 0;
    let flag: i32 = match buf[0] {
        43 => {
            i += 1;
            1_i32
        }
        45 => {
            i += 1;
            -1_i32
        }
        _ => 1_i32,
    };
    while i < buf.len() {
        res = res * 10 + (buf[i] - b'0') as i32;
        i += 1;
    }
    flag * res
}

pub fn real_from_buffer(buf: &[u8]) -> f32 {
    if buf.is_empty() {
        return 0_f32;
    }
    let mut i = 0;
    let flag: f32 = match buf[0] {
        43 => {
            i += 1;
            1_f32
        }
        45 => {
            i += 1;
            -1_f32
        }
        _ => 1_f32,
    };

    let mut ipart = 0_f32;
    while i < buf.len() && buf[i].is_ascii_digit() {
        ipart = ipart * 10_f32 + (buf[i] - b'0') as f32;
        i += 1
    }
    if i < buf.len() && buf[i] != b'.' {
        return flag * ipart;
    } else if i < buf.len() && buf[i] == b'.' {
        i += 1;
        let mut dpart = 0_f32;
        let mut n = 1_f32;
        while i < buf.len() && buf[i].is_ascii_digit() {
            n *= 10_f32;
            dpart = dpart * 10_f32 + (buf[i] - b'0') as f32;
            i += 1
        }
        return flag * (ipart + dpart / n);
    }

    flag * ipart
}
