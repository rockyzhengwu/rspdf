use std::collections::HashMap;

use crate::character::{u16_from_buffer, u32_from_buffer, usize_from_buffer};
use crate::error::{PdfError, Result};
use crate::object::{
    array::PdfArray, dictionary::PdfDict, number::PdfNumber, stream::PdfStream, ObjectId, PdfObject,
};
use crate::reader::PdfReader;
use crate::security::SecurityHandler;

#[derive(Debug)]
pub enum ObjectState {
    Normal,
    Free,
    Compressed,
}

#[derive(Debug)]
pub struct ObjectInfo {
    offset: usize,
    state: ObjectState,
    gen: u16,
}

impl ObjectInfo {
    pub fn new(offset: usize, state: ObjectState, gen: u16) -> Self {
        Self { offset, state, gen }
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn gen(&self) -> u16 {
        self.gen
    }
}

#[derive(Debug)]
pub struct Xref {
    reader: PdfReader,
    objects: HashMap<u32, ObjectInfo>,
    trailer: PdfDict,
    security_handler: Option<SecurityHandler>,
}

impl Xref {
    pub fn try_new(reader: PdfReader, password: Option<&[u8]>) -> Result<Self> {
        let start_xref = find_start_xref(&reader)?;
        reader.reset_offset(start_xref);
        let v = reader.read_token()?;
        if !v.is_other_key(b"startxref") {
            return Err(PdfError::File("read startxref failed".to_string()));
        }
        let xref_offset = reader.read_number()?;
        reader.reset_offset(xref_offset.integer() as usize);
        let nb = reader.read_token()?;
        let mut xref = if nb.is_other_key(b"xref") {
            read_xref_v4(reader, xref_offset)?
        } else {
            read_xref_v5(reader, xref_offset)?
        };

        if let Some(encrypt) = xref.trailer().get("Encrypt") {
            let encrypt = xref.read_object(encrypt)?.to_dict()?;
            let sec = SecurityHandler::try_new(&encrypt, &xref, password)?;
            xref.security_handler = Some(sec);
        }
        Ok(xref)
    }

    pub fn read_object(&self, obj: &PdfObject) -> Result<PdfObject> {
        match obj {
            PdfObject::Indirect(object_id) => {
                let o = self.read_indirect_object(object_id)?;
                return Ok(o);
            }
            _ => Ok(obj.to_owned()),
        }
    }
    pub fn read_indirect_object(&self, objectid: &ObjectId) -> Result<PdfObject> {
        match self.objects.get(&objectid.0) {
            Some(info) => match info.state {
                ObjectState::Normal | ObjectState::Free => {
                    let obj = self.reader.read_indirect_object(info.offset)?;
                    let nt = self.reader.peek_token()?;
                    if nt.is_other_key(b"stream") {
                        match obj {
                            PdfObject::Dict(dict) => match dict.get("Length") {
                                Some(PdfObject::Number(n)) => {
                                    let data = self.reader.read_stream_data(n.integer() as usize)?;
                                    let o = PdfObject::Stream(PdfStream::new(dict, data));
                                    if let Some(secu) = self.security_handler.as_ref() {
                                        let o = secu.decrypt_object(&o, objectid.0, objectid.1)?;
                                        return Ok(o);
                                    } else {
                                        return Ok(o);
                                    }
                                }
                                Some(PdfObject::Indirect(lo)) => {
                                    let cur = self.reader.current_pos();
                                    let len = self
                                        .read_indirect_object(lo)?
                                        .as_number()
                                        .map_err(|_| {
                                            PdfError::Xref(
                                                "PdfStreem Length is not a number".to_string(),
                                            )
                                        })?
                                        .integer();
                                    self.reader.reset_offset(cur);
                                    let data = self.reader.read_stream_data(len as usize)?;
                                    let stream = PdfStream::new(dict, data);
                                    let o = PdfObject::Stream(stream);
                                    if let Some(secu) = self.security_handler.as_ref() {
                                        let o = secu.decrypt_object(&o, objectid.0, objectid.1)?;
                                        return Ok(o);
                                    } else {
                                        return Ok(o);
                                    }
                                }
                                _ => {
                                    return Err(PdfError::Xref(
                                        "PdfStream Length is error".to_string(),
                                    ));
                                }
                            },
                            _ => {
                                return Ok(obj);
                            }
                        }
                    } else {
                        return Ok(obj);
                    }
                }
                ObjectState::Compressed => self.read_objects_stream(info),
            },
            None => Err(PdfError::Xref(format!("Objedct not found:{:?}", objectid))),
        }
    }

    pub fn objects_num(&self) -> usize {
        self.objects.len()
    }
    fn read_objects_stream(&self, info: &ObjectInfo) -> Result<PdfObject> {
        // TODO cache
        let objects_stream = self
            .read_indirect_object(&(info.offset as u32, 0))?
            .to_stream()?;
        let decode_data = objects_stream.decode_data(None)?;
        let reader = PdfReader::new(decode_data);
        let n = objects_stream
            .get_from_dict("N")
            .ok_or(PdfError::Xref("Objects Stream N is None".to_string()))?
            .integer()
            .map_err(|_| PdfError::Xref("N is not number in Objects Stream".to_string()))?;
        let first = objects_stream
            .get_from_dict("First")
            .ok_or(PdfError::Xref("Objects stream First is None".to_string()))?
            .integer()? as usize;

        let tn = (n * 2) as usize;
        let mut ns = Vec::new();
        for _ in 0..tn {
            let v = reader.read_number()?;
            ns.push(v.integer());
        }
        for (i, v) in ns.chunks(2).enumerate() {
            if i == info.gen as usize {
                reader.reset_offset(first + v[1] as usize);
                return reader.read_object();
            }
        }
        return Err(PdfError::Xref(format!(
            "{:?} not found in objects stream",
            info
        )));
    }

    pub fn trailer(&self) -> &PdfDict {
        &self.trailer
    }

    pub fn merge(&mut self, other: Self) {
        self.objects.extend(other.objects);
    }

    pub fn prev(&self) -> Option<i32> {
        match self.trailer().get("Prev") {
            Some(PdfObject::Number(n)) => Some(n.integer()),
            _ => None,
        }
    }

    pub fn reset_offset(&self, offset: usize) {
        self.reader.reset_offset(offset)
    }

    pub fn read_line(&self) -> Result<&[u8]> {
        self.reader.read_line()
    }
}

fn find_start_xref(reader: &PdfReader) -> Result<usize> {
    let len = reader.size().min(4096);
    let pos = reader.size() - len;
    reader.reset_offset(pos);
    let tag = "startxref".as_bytes();
    let mut cur = 0;
    let mut i = 0;
    while i < len {
        if cur == tag.len() {
            break;
        }
        let ch = reader.read_byte()?;
        if ch == &tag[cur] {
            cur += 1;
        } else if ch == &tag[0] {
            cur = 1;
        } else {
            cur = 0
        }
        i += 1;
    }
    let startxref = pos + i - tag.len();
    Ok(startxref)
}

fn read_xref_v4(reader: PdfReader, offset: PdfNumber) -> Result<Xref> {
    let (mut objects, trailer) = read_xref_section(&reader, offset.integer() as usize)?;
    if let Some(p) = trailer.get("Prev") {
        let mut visited = Vec::new();
        visited.push(offset.integer());
        let mut prev_offset = p.integer()?;
        loop {
            if visited.contains(&prev_offset) {
                break;
            }
            let (objs, tr) = read_xref_section(&reader, prev_offset as usize)?;
            objects.extend(objs);
            visited.push(prev_offset);
            if let Some(p) = tr.get("Prev") {
                prev_offset = p.integer()?;
            } else {
                break;
            }
        }
    }
    Ok(Xref {
        reader,
        objects,
        trailer,
        security_handler: None,
    })
}

fn read_xref_section(
    reader: &PdfReader,
    offset: usize,
) -> Result<(HashMap<u32, ObjectInfo>, PdfDict)> {
    reader.reset_offset(offset);
    reader.skip_white_space()?;
    let xref_token = reader.read_token()?;
    let mut objects = HashMap::new();
    if xref_token.is_other_key(b"xref") {
        loop {
            let start = u32_from_buffer(reader.read_token()?.buffer().unwrap())?;
            let count = u32_from_buffer(reader.read_token()?.buffer().unwrap())?;
            let entries = read_xref_table(reader, start, count)?;
            objects.extend(entries);
            reader.skip_white_space()?;
            let next = reader.peek_bytes(7)?;
            if next == b"trailer" {
                reader.read_bytes(7)?;
                reader.skip_white_space()?;
                let _start_dict_token = reader.read_token()?;
                let trailer = reader.read_dict()?;
                return Ok((objects, trailer));
            }
        }
    }
    Err(PdfError::File(
        "pdf need xref keyword in cross-table ".to_string(),
    ))
}

fn read_xref_table(reader: &PdfReader, start: u32, count: u32) -> Result<HashMap<u32, ObjectInfo>> {
    reader.skip_white_space()?;

    let mut objects = HashMap::new();
    for i in 0..count {
        let line = reader.read_bytes(20)?;
        let offset = usize_from_buffer(&line[0..10])?;
        let gen = u16_from_buffer(&line[11..16])?;
        let n = start + i;
        let state = match line[17] {
            b'f' => ObjectState::Free,
            b'n' => ObjectState::Normal,
            _ => {
                return Err(PdfError::ParseObject(format!(
                    "xref table state must be n or f got :{}",
                    line[17]
                )));
            }
        };
        objects.insert(n, ObjectInfo::new(offset, state, gen));
    }
    Ok(objects)
}

fn read_xref_v5(reader: PdfReader, offset: PdfNumber) -> Result<Xref> {
    reader.reset_offset(offset.integer() as usize);
    let _obj_num = reader.read_token()?;
    let _obj_version = reader.read_token()?;
    let _obj_start = reader.read_token()?;
    assert!(_obj_start.is_other_key(b"obj"));
    let _dict_start = reader.read_token()?;
    let st = reader.read_stream()?;
    let mut objects = parse_xref_stream(&reader, &st)?;
    let trailer = st.dict().to_owned();

    if let Some(prev) = trailer.get("Prev") {
        let mut prev_offset = prev.integer()?;
        let mut visited = Vec::new();
        visited.push(offset.integer());
        loop {
            if visited.contains(&prev_offset) {
                break;
            }
            visited.push(prev_offset);
            let (obs, trailer) = read_xref_section(&reader, prev_offset as usize)?;
            objects.extend(obs);
            visited.push(prev_offset);
            if let Some(p) = trailer.get("Prev") {
                prev_offset = p.integer()?;
            } else {
                break;
            }
        }
    }
    Ok(Xref {
        reader,
        objects,
        trailer,
        security_handler: None,
    })
}

fn parse_xref_stream(reader: &PdfReader, stream: &PdfStream) -> Result<HashMap<u32, ObjectInfo>> {
    let wobj: &PdfArray = stream
        .get_from_dict("W")
        .ok_or(PdfError::Object("W dos'nt in xref stream".to_string()))?
        .as_array()?;

    let size = stream
        .get_from_dict("Size")
        .ok_or(PdfError::Object("Size dos'nt in xref stream".to_string()))?
        .as_number()?
        .integer();

    let mut w = Vec::new();
    for v in wobj.iter() {
        w.push(v.as_number()?.integer());
    }
    let index = match stream.get_from_dict("Index") {
        Some(a) => {
            let a = a.as_array()?;
            let mut res = Vec::new();
            for v in a.iter() {
                res.push(v.integer()?);
            }
            res
        }
        None => vec![0, size],
    };

    let buffer = stream.decode_data(None)?;
    let mut entries = HashMap::new();
    let mut bptr = 0;
    for v in index.chunks(2) {
        let start = v[0];
        let length = v[1];
        for num in start..(start + length) {
            let t = if w[0] > 0 {
                let mut t = 0_u32;
                for _ in 0..w[0] {
                    t = (t << 8) + buffer[bptr] as u32;
                    bptr += 1;
                }
                t
            } else {
                1_u32
            };

            let mut offset = 0;
            for _ in 0..w[1] {
                offset = (offset << 8) + buffer[bptr] as usize;
                bptr += 1;
            }
            let mut gen = 0;
            for _ in 0..w[2] {
                gen = (gen << 8) + buffer[bptr] as u16;
                bptr += 1;
            }
            match t {
                0 => {
                    entries.insert(num as u32, ObjectInfo::new(offset, ObjectState::Free, gen));
                }
                1 => {
                    entries.insert(
                        num as u32,
                        ObjectInfo::new(offset, ObjectState::Normal, gen),
                    );
                }
                2 => {
                    entries.insert(
                        num as u32,
                        ObjectInfo::new(offset, ObjectState::Compressed, gen),
                    );
                }
                _ => {
                    return Err(PdfError::Xref(format!(
                        "Xref Entry type must 1,2 or 3 got :{}",
                        t
                    )));
                }
            }
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::find_start_xref;
    use super::read_xref_section;
    use crate::reader::PdfReader;
    fn new_reader(buffer: &str) -> PdfReader {
        PdfReader::new(buffer.as_bytes().to_vec())
    }
    #[test]
    fn test_find_start_xref() {
        let buffer = r#" end obj \r\n startxref 1000 \r\n %EOF "#;
        let reader = new_reader(buffer);
        let startxref = find_start_xref(&reader).unwrap();
        reader.reset_offset(startxref);
        let tag = reader.read_bytes(9).unwrap();
        assert_eq!(tag, "startxref".as_bytes());
    }

    #[test]
    fn test_parse_xref_table() {
        let buffer = b"xref\r\n0 6\r\n0000000003 65535 f\r\n0000000017 00000 n\r\n0000000081 00000 n\r\n0000000000 00007 f\r\n0000000331 00000 n\r\n0000000409 00000 n\r\ntrailer<</Root 1 0 R>>";
        let reader = PdfReader::new(buffer.to_vec());
        let (xref, trailer) = read_xref_section(&reader, 0).unwrap();
        assert_eq!(xref.get(&(4)).unwrap().offset(), 331_usize);

        let buffer = b"xref\r\n0 1\r\n0000000000 65535 f\r\n3 1\r\n0000025325 00000 n\r\n23 2\r\n0000025518 00002 n\r\n0000025635 00000 n\r\n30 1\r\n0000025777 00000 n\r\ntrailer<</Root 1 0 R>>";
        let reader = PdfReader::new(buffer.to_vec());
        let (xref_table, trailer) = read_xref_section(&reader, 0).unwrap();
        assert_eq!(xref_table.len(), 5);
        assert_eq!(xref_table.get(&(24)).unwrap().offset(), 25635);
    }
}
