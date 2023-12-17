use crate::font::cmap::CMap;

pub fn identity_h() -> CMap {
    let bytes = include_bytes!("../../../cmaps/Identity-H");
    CMap::new_from_bytes(bytes)
}
