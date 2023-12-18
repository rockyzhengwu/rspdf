use std::cell::RefCell;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TrueTypeTalbe {
    tag: u32,
    checksum: u32,
    offset: u32,
    len: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TrueTypeCmap {
    platform: u16,
    encoding: u16,
    offset: u32,
    len: u16,
    fmt: u16,
}

#[derive(Debug, Clone)]
pub struct CIDFont {
    reader: RefCell<Cursor<Vec<u8>>>,
    tables: Vec<TrueTypeTalbe>,
    cmaps: Vec<TrueTypeCmap>,
}
#[allow(dead_code)]
impl CIDFont {
    pub fn new(buf: Vec<u8>) -> Self {
        let mut font = CIDFont {
            reader: RefCell::new(Cursor::new(buf)),
            tables: Vec::new(),
            cmaps: Vec::new(),
        };
        font.parse();
        font
    }

    fn lookup_table(&self, tag: &[u8; 4]) -> Option<&TrueTypeTalbe> {
        let tag = u32::from_be_bytes(tag.to_owned());
        self.tables.iter().find(|&t| t.tag == tag)
    }

    fn read_be_u32(&self) -> u32 {
        let mut buf: [u8; 4] = [0; 4];
        // TODO fix
        let n = self.reader.borrow_mut().read(&mut buf).unwrap();
        assert_eq!(n, 4);
        u32::from_be_bytes(buf)
    }

    fn read_be_u16(&self) -> u16 {
        let mut buf: [u8; 2] = [0; 2];
        let n = self.reader.borrow_mut().read(&mut buf).unwrap();
        assert_eq!(n, 2);
        u16::from_be_bytes(buf)
    }

    fn read_u8(&self) -> u8 {
        let mut buf: [u8; 1] = [0; 1];
        let n = self.reader.borrow_mut().read(&mut buf).unwrap();
        assert_eq!(n, 1);
        buf[0]
    }

    fn seek(&self, pos: u64) {
        self.reader.borrow_mut().seek(SeekFrom::Start(pos)).unwrap();
    }

    fn parse(&mut self) {
        self.parse_tables();
        self.parse_cmaps();
        self.parse_post_table();
        println!("parse_tables");
    }

    pub fn map_code_gid(&self, code: u32) -> u32 {
        if self.cmaps.is_empty() {
            return 0;
        }
        let cmap = &self.cmaps[0];
        match cmap.fmt {
            0 => {
                self.seek((cmap.offset + 6 + code) as u64);
                let gid = self.read_u8() as u32;
                return gid;
            }
            2 => {}
            _ => {
                println!("not implemented")
            }
        }
        println!("cmap:{:?}", cmap);

        0
    }

    fn parse_post_table(&mut self) {
        if let Some(post) = self.lookup_table(b"post") {
            self.seek(post.offset as u64);
            let _fmt = self.read_be_u32();
        }
    }

    fn parse_tables(&mut self) {
        let _top_tag = self.read_be_u32();
        // 2
        let ntables = self.read_be_u16();
        // just read
        self.read_be_u32();
        self.read_be_u16();

        for _ in 0..ntables {
            let tag = self.read_be_u32();
            let checksum = self.read_be_u32();
            let offset = self.read_be_u32();
            let len = self.read_be_u32();
            self.tables.push(TrueTypeTalbe {
                tag,
                checksum,
                offset,
                len,
            });
        }
    }

    fn parse_cmaps(&mut self) {
        let mut cmaps = Vec::new();
        if let Some(ct) = self.lookup_table(b"cmap") {
            self.seek((ct.offset + 2) as u64);
            let ncmaps = self.read_be_u16();
            for _ in 0..ncmaps {
                let platform = self.read_be_u16();
                let encoding = self.read_be_u16();
                let offset = self.read_be_u32() + ct.offset;
                let fmt = self.read_be_u16();
                let len = self.read_be_u16();
                let cm = TrueTypeCmap {
                    platform,
                    encoding,
                    offset,
                    fmt,
                    len,
                };
                cmaps.push(cm);
            }
        }
        self.cmaps = cmaps;
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    use super::CIDFont;

    #[test]
    fn teset_parse() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests/resources/cid_opentype.otf");
        let mut file = File::open(d).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let font = CIDFont::new(buffer);
        assert_eq!(font.map_code_gid(109), 79);
    }
}
