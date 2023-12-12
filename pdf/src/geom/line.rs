use crate::geom::point::Point;
use crate::geom::subpath::Segment;

#[derive(Debug, Default)]
pub struct Line {
    start: Point,
    end: Point,
}

impl Segment for Line {
    fn display(&self) -> String {
        format!("Line: start:{:?} end:{:?}", self.start, self.end)
    }
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Line { start, end }
    }
}
