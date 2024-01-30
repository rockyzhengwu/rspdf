use crate::geom::{point::Point, subpath::Segment};

#[derive(Debug, Default)]
pub struct Bezier {
    points: Vec<Point>,
}

impl Segment for Bezier {
    fn display(&self) -> String {
        format!("Bezier : points:{:?}", self.points)
    }

    fn dump_xml(&self) -> String {
        format!(
            "<bezier x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" x3=\"{}\" y3=\"{}\" />\n",
            self.points[0].x(),
            self.points[0].y(),
            self.points[1].x(),
            self.points[1].y(),
            self.points[2].x(),
            self.points[2].y()
        )
    }
}

impl Bezier {
    pub fn new(points: Vec<Point>) -> Self {
        Bezier { points }
    }

    pub fn points(&self) -> &[Point] {
        self.points.as_slice()
    }
}
