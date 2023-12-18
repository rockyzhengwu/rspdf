use crate::font::cmap::CMap;

use crate::font::cmap::identity_h::identity_h;

pub fn get_predefine_cmap(name: &str) -> CMap {
    match name {
        "Identity-H" => identity_h(),
        _ => {
            panic!("not match")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::get_predefine_cmap;

    #[test]
    fn test_get_predefine_cmap() {
        let name = "Identity-H";
        let cmap = get_predefine_cmap(name);
        println!("{:?}", cmap);
    }
}
