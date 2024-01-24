/* ToUnicode methods:
   according PDF 3200 9.10.2 only three condition can extract unicode from PDF
1. use embedding cmap in ToUnicode
2. Simple font use eodong name to unicode
3. If the font is a composite font that uses one of the predefined CMaps listed in Table 118 (except Identity–H and Identity–V) or whose descendant CIDFont uses the Adobe-GB1, Adobe-CNS1, Adobe-Japan1, or Adobe-Korea1 character collection
*/
use std::collections::HashMap;

use crate::font::cmap::CMap;

// TODO split implement charcode to unicode

pub struct CharCodeToUnicode {
    char_to_unicode: HashMap<u32, u32>,
    // a cmap foramt unicode mapping,
    ctu: CMap,
}
