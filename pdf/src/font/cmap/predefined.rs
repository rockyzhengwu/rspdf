use std::collections::HashMap;

use crate::errors::PDFResult;
use crate::font::cmap::CMap;
use lazy_static::lazy_static;

// TODO init by generate

pub fn create_cmap(bytes: &[u8]) -> PDFResult<CMap> {
    CMap::new_from_bytes(bytes)
}

pub fn identity_h() -> &'static [u8] {
    let identity_h_bytes = include_bytes!("../../../cmaps/Identity-H");
    identity_h_bytes
}

pub fn identity_v() -> &'static [u8] {
    let identity_v = include_bytes!("../../../cmaps/Identity-V");
    identity_v
}
pub fn unicns_utf16_h() -> &'static [u8] {
    let unicns_utf16_h = include_bytes!("../../../cmaps/UniCNS-UTF16-H");
    unicns_utf16_h
}
pub fn adobe_cns1_ucs2() -> &'static [u8] {
    let adobe_cns1_ucs2 = include_bytes!("../../../cmaps/Adobe-CNS1-UCS2");
    adobe_cns1_ucs2
}

lazy_static! {
    static ref PREDEFINE_CMAP: HashMap<String, &'static [u8]> = {
        let mut m = HashMap::new();
        m.insert("Identity-H".to_string(), identity_h());
        m.insert("Identity-V".to_string(), identity_v());
        m.insert("Adobe-CNS1-UCS2".to_string(), adobe_cns1_ucs2());
        m
    };
}

pub fn get_predefine_cmap(name: &str) -> CMap {
    create_cmap(PREDEFINE_CMAP.get(name).unwrap().to_owned()).unwrap()
}

#[cfg(test)]
mod tests {

    use super::get_predefine_cmap;

    #[test]
    fn test_get_predefine_cmap() {
        let name = "Identity-V";
        let cmap = get_predefine_cmap(name);
        let bytes: Vec<u8> = vec![53, 53, 56, 70, 50, 56, 51, 65, 52, 70, 57, 56];
        let char = cmap.next_char(bytes.as_slice(), 0).unwrap();
        let cid = cmap.charcode_to_cid(&char);
        println!("{:?},{:?}", char, cid);
    }
}
