use crate::geom::{point::Point, subpath::Segment};

#[derive(Debug, Default)]
pub struct Bezier {
    points: Vec<Point>,
}

impl Segment for Bezier {
    fn display(&self) -> String {
        format!("Bezier : points:{:?}", self.points)
    }
}

impl Bezier {
    pub fn new(points: Vec<Point>) -> Self {
        Bezier { points }
    }
}
