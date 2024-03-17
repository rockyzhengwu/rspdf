use crate::geom::bezier::Bezier;
use crate::geom::line::Line;
use crate::geom::point::Point;
use crate::geom::rectangle::Rectangle;

#[derive(Debug)]
pub enum Segment {
    Rect(Rectangle),
    Line(Line),
    Curve(Bezier),
}

#[derive(Default, Debug)]
pub struct SubPath {
    start: Point,
    closed: bool,
    segments: Vec<Segment>,
}

impl SubPath {
    pub fn new(start: Point) -> Self {
        SubPath {
            start,
            closed: false,
            segments: Vec::new(),
        }
    }
    pub fn is_single_point(&self) -> bool {
        self.segments.is_empty() && !self.closed
    }

    pub fn add_segment(&mut self, seg: Segment) {
        self.segments.push(seg);
    }

    pub fn is_slosed(&self) -> bool {
        self.closed
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn segments(&self) -> &[Segment] {
        self.segments.as_slice()
    }

    pub fn set_start(&mut self, start: Point) {
        self.start = start
    }
}
