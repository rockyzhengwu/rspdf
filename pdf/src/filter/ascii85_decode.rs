use std::i64;

use crate::errors::PDFError;
use crate::filter::Filter;

#[derive(Default)]
pub struct ASCII85Decode {}

impl Filter for ASCII85Decode {
    fn decode(
        &self,
        buf: &[u8],
        _param: Option<crate::object::PDFDictionary>,
    ) -> crate::errors::PDFResult<Vec<u8>> {
        let mut res: Vec<u8> = Vec::new();
        let mut chunk: [u8; 5] = [0; 5];
        let mut pos: usize = 0;
        for c in buf.iter() {
            if *c == b'~' {
                break;
            }
            if *c == b'z' && pos == 0 {
                res.push(0);
                res.push(0);
                res.push(0);
                res.push(0);
                continue;
            }
            if *c < b'!' || *c > b'u' {
                return Err(PDFError::Filter(format!(
                    "{:?} not a valid character in ASCII85",
                    c
                )));
            }
            chunk[pos] = *c - 33;
            pos += 1;
            if pos == 5 {
                let mut r: i64 = 0;
                for v in chunk {
                    r = r * 85 + v as i64;
                }
                res.push(((r >> 24) & 0xff) as u8);
                res.push(((r >> 16) & 0xff) as u8);
                res.push(((r >> 8) & 0xff) as u8);
                res.push((r & 0xff) as u8);
                pos = 0;
            }
        }
        for v in chunk.iter_mut().skip(pos) {
            *v = 84_u8;
        }
        let mut r: i64 = 0;
        for v in chunk {
            r = r * 85 + v as i64;
        }
        match pos {
            2 => res.push((r >> 24 & 0xff) as u8),
            3 => {
                res.push((r >> 24 & 0xff) as u8);
                res.push((r >> 16 & 0xff) as u8);
            }
            4 => {
                res.push((r >> 24 & 0xff) as u8);
                res.push((r >> 16 & 0xff) as u8);
                res.push((r >> 8 & 0xff) as u8);
            }
            _ => {
                // Do Nothing
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {

    use super::ASCII85Decode;
    use crate::filter::Filter;

    #[test]
    fn test_decode() {
        let content = r#"9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c"#;
        let decoder = ASCII85Decode::default();
        let result = decoder.decode(content.as_bytes(), None).unwrap();
        let res = String::from_utf8(result).unwrap();
        let expected = r#"Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure."#;
        assert_eq!(res, expected);
    }
}
