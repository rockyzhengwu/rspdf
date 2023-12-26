use std::collections::HashMap;

use crate::font::cmap::CMap;
use lazy_static::lazy_static;

// TODO init by generate

pub fn create_cmap(bytes: &[u8]) -> CMap {
    CMap::new_from_bytes(bytes)
}

pub fn identity_h() -> CMap {
    let bytes = include_bytes!("../../../cmaps/Identity-H");
    create_cmap(bytes)
}

pub fn identity_v() -> CMap {
    let bytes = include_bytes!("../../../cmaps/Identity-V");
    create_cmap(bytes)
}

lazy_static! {
    static ref PREDEFINE_CMAP: HashMap<String, CMap> = {
        let mut m = HashMap::new();
        m.insert("Identity-H".to_string(), identity_h());
        m.insert("Identity-V".to_string(), identity_v());
        m
    };
}

pub fn get_predefine_cmap(name: &str) -> CMap {
    PREDEFINE_CMAP.get(name).unwrap().to_owned()
}

pub fn get_predefine_cmap_ref(name: &str) -> &CMap {
    PREDEFINE_CMAP.get(name).unwrap()
}

#[cfg(test)]
mod tests {

    use super::get_predefine_cmap;

    #[test]
    fn test_get_predefine_cmap() {
        let name = "Identity-H";
        let cmap = get_predefine_cmap(name);
        println!("{:?}", cmap);
        println!("{:?}", cmap.code_to_cid(vec![0, 36].as_slice()));
    }
}
