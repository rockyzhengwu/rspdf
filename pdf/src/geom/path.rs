use crate::geom::bezier::Bezier;
use crate::geom::line::Line;
use crate::geom::point::Point;
use crate::geom::rectangle::Rectangle;
use crate::geom::subpath::Segment;
use crate::geom::subpath::SubPath;

#[derive(Debug, Default, Clone)]
pub struct Path {
    current: Point,
    sub_paths: Vec<SubPath>,
}

impl Path {
    pub fn new(point: Point) -> Self {
        Self {
            current: point,
            sub_paths: Vec::new(),
        }
    }

    pub fn move_to(&mut self, point: Point) {
        self.current = point;
        if let Some(sub) = self.sub_paths.last_mut() {
            if sub.is_single_point() {
                sub.set_start(self.current);
            }
        } else {
            self.sub_paths.push(SubPath::new(point));
        }
    }

    pub fn curve_to(&mut self, mut points: Vec<Point>) {
        let n = 4 - points.len();
        for _ in 0..n {
            points.insert(0, self.current);
        }
        self.current = points.last().unwrap().to_owned();
        let bezier = Bezier::new(points);
        self.sub_paths
            .last_mut()
            .unwrap()
            .add_segment(Segment::Curve(bezier));
    }

    pub fn line_to(&mut self, target: Point) {
        if let Some(l) = self.sub_paths.last_mut() {
            l.add_segment(Segment::Line(Line::new(self.current, target)));
        }
        self.current = target;
    }

    // x y m
    //  ( x + width ) y l
    //  ( x + width ) ( y + height ) l
    //  x ( y + height ) l
    pub fn rectangle(&mut self, rect: Rectangle) {
        self.move_to(rect.lower_left().to_owned());
        self.line_to(Point::new(rect.ux(), rect.ly()));
        self.line_to(Point::new(rect.ux(), rect.uy()));
        self.line_to(Point::new(rect.lx(), rect.uy()));
        self.close_last_subpath();
    }

    pub fn close_last_subpath(&mut self) {
        if let Some(v) = self.sub_paths.last_mut() {
            v.close()
        }
    }

    pub fn current_point(&self) -> &Point {
        &self.current
    }

    pub fn sub_paths(&self) -> &[SubPath] {
        self.sub_paths.as_slice()
    }
}
