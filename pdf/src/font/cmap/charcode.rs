#[derive(Debug)]
pub struct CharCode {
    code: u32,
    length: u8,
}

impl CharCode {
    pub fn new(code: u32, length: u8) -> Self {
        CharCode { code, length }
    }

    pub fn code(&self) -> u32 {
        self.code
    }

    pub fn length(&self) -> u8 {
        self.length
    }
}
