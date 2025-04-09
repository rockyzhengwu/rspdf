use super::bitreader::BitReader;
use super::fax_table::{decode_mode, decode_run_length, Color, Mode};
use super::Param;
use crate::error::{PdfError, Result};

pub struct Group4Decoder<'a> {
    reader: BitReader<'a>,
    width: u16,
    reference: Vec<u16>,
    current: Vec<u16>,
}

pub struct RefState<'a> {
    pos: usize,
    changes: &'a [u16],
}

impl<'a> RefState<'a> {
    pub fn new(changes: &'a [u16]) -> Self {
        RefState { changes, pos: 0 }
    }

    pub fn step_back(&mut self, end: u16) {
        while self.pos > 0 {
            if self.changes[self.pos] >= end {
                self.pos -= 1;
                continue;
            } else {
                break;
            }
        }
    }

    pub fn next_change(&mut self, color: Color, start: u16) -> Option<u16> {
        if self.changes.is_empty() || self.pos >= self.changes.len() {
            return None;
        }

        while let Some(p) = self.changes.get(self.pos) {
            if p > &start {
                break;
            } else {
                self.pos += 1;
            }
        }

        //while self.pos < self.changes.len() {
        //    if self.changes[self.pos] < start {
        //        self.pos += 1;
        //        continue;
        //    } else {
        //        break;
        //    }
        //}
        match color {
            Color::Black => {
                if self.pos % 2 == 0 {
                    return Some(self.pos as u16);
                } else {
                    if self.pos + 1 < self.changes.len() {
                        self.pos += 1;
                        return Some(self.pos as u16);
                    } else {
                        return None;
                    }
                }
            }
            Color::White => {
                if self.pos % 2 != 0 {
                    return Some(self.pos as u16);
                } else {
                    if self.pos + 1 < self.changes.len() {
                        self.pos += 1;
                        return Some(self.pos as u16);
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    pub fn move_next(&mut self) -> Option<u16> {
        if self.pos + 1 < self.changes.len() {
            let val = self.changes[self.pos + 1];
            self.pos += 1;
            return Some(val);
        }
        None
    }
}

impl<'a> Group4Decoder<'a> {
    pub fn new(reader: BitReader<'a>, width: u16) -> Self {
        Group4Decoder {
            reader,
            width,
            reference: Vec::new(),
            current: Vec::new(),
        }
    }
    pub fn decode_line(&mut self) -> Result<Vec<u8>> {
        let mut a0 = 0;
        let mut color = Color::White;
        let mut refstate = RefState::new(self.reference.as_slice());
        loop {
            let mode = decode_mode(&mut self.reader).ok_or(PdfError::Filter(
                "Ccittfax Mode is not detected".to_string(),
            ))?;
            println!("{:?},{:?}", mode, a0);
            match mode {
                Mode::Pass => {
                    let _b1 = refstate.next_change(!color, a0).ok_or(PdfError::Filter(
                        "Ccittfax Pass Mode b1 is None".to_string(),
                    ))?;
                    if let Some(b2) = refstate.move_next() {
                        a0 = b2;
                    } else {
                        return Err(PdfError::Filter(
                            "Ccittfax Pass Mode b2 is None".to_string(),
                        ));
                    }
                }
                Mode::Horizontal => {
                    let a0a1 = decode_run_length(&mut self.reader, color).ok_or(
                        PdfError::Filter("Ccittfax runlength decode error".to_string()),
                    )?;
                    let a1a2 = decode_run_length(&mut self.reader, !color).ok_or(
                        PdfError::Filter("Ccittfax runlength decode error".to_string()),
                    )?;
                    let a1 = a0 + a0a1;
                    let a2 = a1 + a1a2;
                    //println!("{:?},{},{},{}", mode, a0a1, a1a2, a2);
                    self.current.push(a1);
                    if a2 >= self.width {
                        break;
                    }
                    self.current.push(a2);
                    a0 = a2;
                }
                Mode::V => {
                    let b1 = refstate.next_change(!color, a0).unwrap_or(self.width);
                    let a1 = b1 + 0;
                    println!("{:?},{:?}", a0, b1);
                    if a1 >= self.width {
                        break;
                    }
                    self.current.push(a1);
                    a0 = a1;
                    color = !color;
                }
                Mode::VR(n) => {
                    let b1 = refstate.next_change(!color, a0).unwrap_or(self.width);
                    let a1 = b1 + n as u16;
                    if a1 >= self.width {
                        break;
                    }
                    self.current.push(a1);
                    a0 = a1;
                    color = !color;
                }
                Mode::VL(n) => {
                    let b1 = refstate.next_change(!color, a0).unwrap_or(self.width);
                    let a1 = b1 - n as u16;
                    if a1 >= self.width {
                        break;
                    }
                    self.current.push(a1);
                    a0 = a1;

                    refstate.step_back(a0);
                    color = !color;
                }
                Mode::End => {
                    return Ok(Vec::new());
                }
            }
        }
        let mut line = Vec::new();
        let mut start = 0_u16;
        for (i, change) in self.current.iter().enumerate() {
            if i % 2 == 0 {
                for _ in start..*change {
                    line.push(255);
                }
            } else {
                for _ in start..*change {
                    line.push(0);
                }
            }
            start = change.to_owned();
        }
        std::mem::swap(&mut self.reference, &mut self.current);
        self.current.clear();
        if line.is_empty() {
            line = vec![255; self.width as usize];
        }
        for _ in line.len()..self.width as usize {
            line.push(0);
        }
        return Ok(line);
    }
}

pub fn decode_g4(buffer: &[u8], param: &Param) -> Result<Vec<u8>> {
    let reader = BitReader::new(buffer);
    let mut decoder = Group4Decoder::new(reader, param.columns);
    let mut res = Vec::new();
    let rows = if param.rows > 0 { param.rows } else { u16::MAX };
    for i in 0..rows {
        let line = decoder.decode_line()?;
        if line.is_empty() {
            break;
        }
        if i > 406 {
            panic!("finish");
        }
        println!("row: {:?},{:?}", i, decoder.reference);
        res.extend(line);
    }

    Ok(res)
}
