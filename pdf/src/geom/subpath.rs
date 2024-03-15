use std::fmt;

use crate::geom::point::Point;

pub trait Segment {
    fn display(&self) -> String;
    fn dump_xml(&self) -> String;
    fn points(&self) -> Vec<Point>;
}

impl fmt::Debug for dyn Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Segment:{:?}", self.display())
    }
}

#[derive(Default, Debug)]
pub struct SubPath {
    start: Point,
    closed: bool,
    segments: Vec<Box<dyn Segment>>,
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

    pub fn add_segment(&mut self, seg: Box<dyn Segment>) {
        self.segments.push(seg);
    }

    pub fn is_slosed(&self) -> bool {
        self.closed
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn segments(&self) -> &[Box<dyn Segment>] {
        self.segments.as_slice()
    }

    pub fn set_start(&mut self, start: Point) {
        self.start = start
    }
}
