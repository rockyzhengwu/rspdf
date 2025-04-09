use std::collections::HashMap;

use crate::error::{PdfError, Result};

#[derive(Debug, Default, Clone)]
pub struct CharMetrics {
    code: i32,
    wx: f32,
    bbox: [f32; 4],
    name: String,
}

#[derive(Debug, Default, Clone)]
pub struct Afm {
    font_name: String,
    family_name: String,
    char_metrics: HashMap<String, CharMetrics>,
}
impl Afm {
    pub fn get_char_width(&self, name: &str) -> Option<f32> {
        if let Some(cm) = self.char_metrics.get(name) {
            return Some(cm.wx);
        }
        None
    }
    pub fn get_char_width_with_code(&self, code: u8) -> Option<f32> {
        for (key, cm) in self.char_metrics.iter() {
            if cm.code == code as i32 {
                return Some(cm.wx);
            }
        }
        None
    }
}

fn parse_line(line: &str) -> Option<(&str, Vec<&str>)> {
    let words: Vec<&str> = line.split_ascii_whitespace().collect();
    if words.is_empty() {
        None
    } else {
        if let Some((key, values)) = words.split_first() {
            return Some((key.to_owned(), values.to_owned()));
        }
        None
    }
}

fn parse_char_metric(line: &str) -> Option<CharMetrics> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    } else {
        let items = line.split(';');
        let mut char_matrics = CharMetrics::default();
        for item in items {
            let item = item.trim();
            if let Some((key, values)) = parse_line(item) {
                match key {
                    "C" => {
                        let code = values.first().unwrap().parse::<i32>().unwrap();
                        char_matrics.code = code;
                    }
                    "WX" => {
                        let wx = values.first().unwrap().parse::<f32>().unwrap();
                        char_matrics.wx = wx;
                    }
                    "N" => {
                        let name = values.first().unwrap().to_string();
                        char_matrics.name = name;
                    }
                    "B" => {
                        let lx = values.first().unwrap().parse::<f32>().unwrap();
                        let ly = values.get(1).unwrap().parse::<f32>().unwrap();
                        let ux = values.get(2).unwrap().parse::<f32>().unwrap();
                        let uy = values.get(3).unwrap().parse::<f32>().unwrap();
                        char_matrics.bbox = [lx, ly, ux, uy];
                    }
                    _ => {}
                }
            }
        }
        return Some(char_matrics);
    }
}

pub fn parse_afm(content: String) -> Result<Afm> {
    let mut lines = content.split('\n');
    let mut afm = Afm::default();
    loop {
        match lines.next() {
            Some(line) => {
                if let Some((key, values)) = parse_line(line) {
                    match key {
                        "FontName" => {
                            let fname = values.first().unwrap().to_string();
                            afm.font_name = fname;
                        }
                        "FamilyName" => {
                            let fname = values.first().unwrap().to_string();
                            afm.family_name = fname;
                        }
                        "CharWidth" => {}
                        "StartCharMetrics" => {
                            let n = values
                                .first()
                                .ok_or(PdfError::Font(
                                    "Afm charactor metrics number is none".to_string(),
                                ))?
                                .parse::<u32>()
                                .unwrap();
                            let mut i = 0;
                            while i < n {
                                if let Some(nl) = lines.next() {
                                    if let Some(char_matrics) = parse_char_metric(nl) {
                                        afm.char_metrics
                                            .insert(char_matrics.name.to_owned(), char_matrics);
                                        i += 1;
                                    }
                                } else {
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            None => {
                break;
            }
        }
    }
    Ok(afm)
}

#[cfg(test)]
mod tests {
    use font_data::get_builtin_font_matrices;

    use super::parse_afm;

    #[test]
    fn test_afm() {
        let afm_data = get_builtin_font_matrices("Courier").unwrap();
        let s = String::from_utf8(afm_data.to_vec()).unwrap();
        let afm = parse_afm(s).unwrap();
        assert_eq!(315, afm.char_metrics.len());
    }
}
