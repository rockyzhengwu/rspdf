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

    fn dump_xml(&self) -> String {
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />\n",
            self.start.x(),
            self.start.y(),
            self.end.x(),
            self.end.y()
        )
    }

    fn points(&self) -> Vec<Point> {
        vec![self.start.clone(), self.end.clone()]
    }
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Line { start, end }
    }
}
