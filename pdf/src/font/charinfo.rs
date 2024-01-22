
#[derive(Debug)]
pub struct CharInfo {
    len: u8,
    charcode: u32,
    cid: u32,
    unicode: char,
}

impl CharInfo {
    pub fn new(len: u8, charcode: u32, cid: u32, unicode: char) -> Self {
        CharInfo {
            len,
            charcode,
            cid,
            unicode,
        }
    }

    pub fn unicode(&self) -> &char {
        &self.unicode
    }
    pub fn len(&self) -> &u8 {
        &self.len
    }
    pub fn cid(&self) -> &u32 {
        &self.cid
    }

    pub fn charcode(&self) -> &u32 {
        &self.charcode
    }
}
