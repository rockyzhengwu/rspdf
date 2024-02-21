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
    closed: bool,
    segments: Vec<Box<dyn Segment>>,
}

impl SubPath {
    pub fn new(seg: Box<dyn Segment>) -> Self {
        let segments = vec![seg];
        SubPath {
            closed: false,
            segments,
        }
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
}
