#[derive(Debug)]
pub struct CharInfo {
    cid: u32,
    unicode: char,
}

impl CharInfo {
    pub fn new(cid: u32, unicode: char) -> Self {
        CharInfo { cid, unicode }
    }

    pub fn unicode(&self) -> &char {
        &self.unicode
    }

    pub fn cid(&self) -> &u32 {
        &self.cid
    }
}
