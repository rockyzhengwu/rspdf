use crate::font::cmap::CMap;
use font_data::cmap::get_predefine_cmap_data;

pub fn get_predefine_cmap(name: &str) -> Option<CMap> {
    if let Some(cmap_data) = get_predefine_cmap_data(name) {
        let cmap = CMap::new_from_bytes(cmap_data).unwrap();
        return Some(cmap);
    }
    None
}

#[cfg(test)]
mod tests {

    use super::get_predefine_cmap;

    #[test]
    fn test_get_predefine_cmap() {
        let name = "Identity-H";
        let cmap = get_predefine_cmap(name).unwrap();
        let bytes: Vec<u8> = vec![53, 53, 56, 70, 50, 56, 51, 65, 52, 70, 57, 56];
        let char = cmap.next_char(bytes.as_slice(), 0).unwrap();
        let cid = cmap.charcode_to_cid(&char);
        println!("{:?},{:?}", char, cid);
    }
}
