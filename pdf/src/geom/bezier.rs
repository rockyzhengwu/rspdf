use crate::geom::coordinate::Point;

#[derive(Debug, Default, Clone)]
pub struct BezierQuad {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
}

impl BezierQuad {
    pub fn new(p0: Point, p1: Point, p2: Point) -> Self {
        Self { p0, p1, p2 }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BezierCubic {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

impl BezierCubic {
    pub fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        BezierCubic { p0, p1, p2, p3 }
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    Curve3(BezierQuad),
    Curve4(BezierCubic),
}
