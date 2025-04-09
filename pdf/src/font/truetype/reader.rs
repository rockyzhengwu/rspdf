use crate::error::{PdfError, Result};

pub struct TrueTypeReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> TrueTypeReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        TrueTypeReader { data, offset: 0 }
    }

    pub fn read_be_u16(&mut self) -> Result<u16> {
        if self.offset + 2 >= self.data.len() {
            return Err(PdfError::Font("Tretype Program reader eof".to_string()));
        }
        let v: u16 = u16::from_be_bytes(
            self.data[self.offset..self.offset + 2]
                .try_into()
                .map_err(|_| PdfError::Font("TrueTypeReader read u16 error".to_string()))?,
        );
        self.offset += 2;
        Ok(v)
    }

    pub fn read_be_i16(&mut self) -> Result<i16> {
        if self.offset + 2 >= self.data.len() {
            return Err(PdfError::Font("Tretype Program reader eof".to_string()));
        }
        let v: i16 = i16::from_be_bytes(
            self.data[self.offset..self.offset + 2]
                .try_into()
                .map_err(|_| PdfError::Font("TrueTypeReader read i16 error".to_string()))?,
        );
        self.offset += 2;
        Ok(v)
    }

    pub fn read_be_u32(&mut self) -> Result<u32> {
        if self.offset + 4 >= self.data.len() {
            return Err(PdfError::Font(
                "Tretype Program read be_u32 eof".to_string(),
            ));
        }
        let v: u32 = u32::from_be_bytes(
            self.data[self.offset..self.offset + 4]
                .try_into()
                .map_err(|_| PdfError::Font("TrueTypeReader read u16 error".to_string()))?,
        );
        self.offset += 4;
        Ok(v)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.offset >= self.data.len() {
            return Err(PdfError::Font(
                "Tretype Program read be_u32 eof".to_string(),
            ));
        }
        let v = self.data[self.offset];
        self.offset += 1;
        Ok(v)
    }

    pub fn read_tag(&mut self) -> Result<String> {
        if self.offset + 4 >= self.data.len() {
            return Err(PdfError::Font("Tretype Program read tag eof".to_string()));
        }
        let bytes = &self.data[self.offset..self.offset + 4];
        let tag = String::from_utf8(bytes.to_vec())
            .map_err(|_| PdfError::Font("Tretype program read tag error".to_string()))?;
        self.offset += 4;
        Ok(tag)
    }

    pub fn reset_offset(&mut self, offset: usize) {
        self.offset = offset;
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::TrueTypeReader;

    #[test]
    fn test_reader() {
        let buffer = vec![0x12, 0x34, 0x34, 0x12];
        let mut reader = TrueTypeReader::new(buffer.as_slice());
        let u = reader.read_be_u16().unwrap();
        assert_eq!(u, 4660);
        let u = reader.read_u8().unwrap();
        assert_eq!(u, 0x34);
    }
}
