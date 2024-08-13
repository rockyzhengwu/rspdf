use crate::geom::point::Point;

#[derive(Debug, Default,Clone)]
pub struct Line {
    start: Point,
    end: Point,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Line { start, end }
    }
    pub fn start(&self) -> &Point {
        &self.start
    }

    pub fn end(&self) -> &Point {
        &self.end
    }
}
