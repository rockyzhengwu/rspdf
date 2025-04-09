pub struct Type1CProgram {}

struct Type1Parser {
    data: Vec<u8>,
    offset: usize,
}

impl Type1Parser {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, offset: 0 }
    }
}
