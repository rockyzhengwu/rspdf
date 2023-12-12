use crate::geom::bezier::Bezier;
use crate::geom::line::Line;
use crate::geom::point::Point;
use crate::geom::rectangle::Rectangle;
use crate::geom::subpath::SubPath;

#[derive(Debug, Default)]
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
        self.close_last_subpath();
        self.current = point;
    }

    pub fn curve_to(&mut self, mut points: Vec<Point>) {
        let p = std::mem::take(&mut self.current);
        points.insert(0, p);
        self.current = points.last().unwrap().clone();
        let bezier = Bezier::new(points);
        if self.sub_paths.is_empty() {
            self.sub_paths.push(SubPath::new(Box::new(bezier)));
        } else {
            self.sub_paths
                .last_mut()
                .unwrap()
                .add_segment(Box::new(bezier));
        }
    }

    pub fn line_to(&mut self, target: Point) {
        let l = Line::new(self.current.clone(), target.clone());
        if !self.sub_paths.is_empty() && !self.sub_paths.last().unwrap().is_slosed() {
            self.sub_paths.last_mut().unwrap().add_segment(Box::new(l));
        } else {
            self.sub_paths.push(SubPath::new(Box::new(l)));
        }
        self.current = target;
    }

    // x y m
    //  ( x + width ) y l
    //  ( x + width ) ( y + height ) l
    //  x ( y + height ) l
    pub fn rectangle(&mut self, rect: Rectangle) {
        self.move_to(rect.lower_left());
        // lower_right;
        self.line_to(Point::new(rect.x() + rect.width(), rect.y()));
        self.line_to(Point::new(
            rect.x() + rect.width(),
            rect.y() + rect.height(),
        ));
        self.line_to(Point::new(rect.x(), rect.y() + rect.height()));
        self.close_last_subpath();
    }

    pub fn close_last_subpath(&mut self) {
        if let Some(v) = self.sub_paths.last_mut() {
            v.close()
        }
    }
}
