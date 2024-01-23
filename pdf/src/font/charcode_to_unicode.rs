use std::collections::HashMap;

use crate::font::cmap::CMap;

// TODO split implement charcode to unicode

pub struct CharCodeToUnicode {
    char_to_unicode: HashMap<u32, u32>,
    // a cmap foramt unicode mapping,
    ctu: CMap,
}
