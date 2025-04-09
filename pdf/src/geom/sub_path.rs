use crate::{
    error::{PdfError, Result},
    geom::{
        bezier::{BezierCubic, BezierQuad},
        coordinate::Point,
    },
};

#[derive(Debug, Clone)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    Curve3(BezierQuad),
    Curve4(BezierCubic),
    Closed,
}

#[derive(Debug, Default, Clone)]
pub struct SubPath {
    segments: Vec<PathSegment>,
    is_closed: bool,
}
impl SubPath {
    pub fn new(point: Point) -> Self {
        let mut segments = Vec::new();
        segments.push(PathSegment::MoveTo(point));
        SubPath {
            segments,
            is_closed: false,
        }
    }

    pub fn segments(&self) -> &[PathSegment] {
        self.segments.as_slice()
    }

    pub fn close(&mut self) {
        if self.is_closed || self.segments.len() <= 1 {
            return;
        }
        self.segments.push(PathSegment::Closed);
        //match self.segments.first() {
        //    Some(PathSegment::MoveTo(p1)) => self.segments.push(PathSegment::Closed),
        //    _ => {
        //        // donothing
        //    }
        //}
        self.is_closed = true;
    }

    pub fn move_to(&mut self, point: Point) {
        if self.segments.len() == 1 {
            self.segments.clear();
            self.segments.push(PathSegment::MoveTo(point));
        } else {
            self.segments.push(PathSegment::MoveTo(point));
        }
    }

    pub fn line_to(&mut self, point: Point) -> Result<()> {
        if self.segments.is_empty() {
            return Err(PdfError::Path(
                "Subpath lineto , subpath is empty".to_string(),
            ));
        }
        self.segments.push(PathSegment::LineTo(point));
        Ok(())
    }

    pub fn curve_cubic(&mut self, curve: BezierCubic) {
        self.segments.push(PathSegment::Curve4(curve));
    }

    pub fn curve_quad(&mut self, curve: BezierQuad) {
        self.segments.push(PathSegment::Curve3(curve));
    }

    pub fn add_segment(&mut self, seg: PathSegment) {
        self.segments.push(seg)
    }

    pub fn start_point(&self) -> Option<&Point> {
        if let Some(seg) = self.segments.first() {
            match seg {
                PathSegment::MoveTo(p) => Some(p),
                PathSegment::Curve3(curve) => Some(&curve.p1),
                _ => None,
            }
        } else {
            None
        }
    }
    pub fn is_single_move(&self) -> bool {
        self.segments.len() == 1
    }
}
