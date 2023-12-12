#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Color {
    Gray(f64),
    Rgb(f64, f64, f64),
    Cmyk(f64, f64, f64, f64),
}
