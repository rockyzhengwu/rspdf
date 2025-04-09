pub struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    pub fn read_bit(&mut self) -> Option<u8> {
        if self.bit_pos / 8 >= self.data.len() {
            return None;
        }
        let byte = self.data[self.bit_pos / 8];
        let bit = (byte >> (7 - (self.bit_pos % 8))) & 1;
        self.bit_pos += 1;
        Some(bit)
    }

    pub fn read_bits(&mut self, count: usize) -> Option<u32> {
        let mut value = 0;
        for _ in 0..count {
            value = (value << 1) | self.read_bit()? as u32;
        }
        Some(value)
    }

    pub fn remaining_bits(&self) -> usize {
        if self.bit_pos > self.data.len() {
            return 0;
        }
        self.data.len() - self.bit_pos
    }

    pub fn eof(&self) -> bool {
        self.bit_pos / 8 >= self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::BitReader;

    #[test]
    fn test_bit_reader() {
        let buf: Vec<u8> = vec![72, 137];
        let mut reader = BitReader::new(buf.as_slice());
        let v = reader.read_bits(4);
        println!("{:?}", v);
    }
}
