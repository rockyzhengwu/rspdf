use crate::geom::point::Point;

#[derive(Debug, Default)]
pub struct Bezier {
    points: Vec<Point>,
}

impl Bezier {
    pub fn new(points: Vec<Point>) -> Self {
        Bezier { points }
    }
    pub fn points(&self) -> &[Point] {
        self.points.as_slice()
    }
}
