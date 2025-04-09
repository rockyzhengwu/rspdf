use crate::{
    error::{PdfError, Result},
    geom::{
        bezier::{BezierCubic, BezierQuad},
        coordinate::Point,
        rect::Rect,
        sub_path::{PathSegment, SubPath},
    },
};

#[derive(Default, Debug, Clone)]
pub struct Path {
    subpaths: Vec<SubPath>,
}

impl Path {
    pub fn move_to(&mut self, point: Point) {
        self.subpaths.push(SubPath::new(point))
    }

    pub fn line_to(&mut self, point: Point) -> Result<()> {
        if let Some(last) = self.subpaths.last_mut() {
            last.add_segment(PathSegment::LineTo(point));
            Ok(())
        } else {
            Err(PdfError::Path("Lineto Path is empty".to_string()))
        }
    }

    pub fn subpaths(&self) -> &[SubPath] {
        self.subpaths.as_slice()
    }

    pub fn curve4(&mut self, p0: Point, p1: Point, p2: Point, p3: Point) -> Result<()> {
        if let Some(last) = self.subpaths.last_mut() {
            last.add_segment(PathSegment::Curve4(BezierCubic::new(p0, p1, p2, p3)));
            Ok(())
        } else {
            Err(PdfError::Path("Curve4 Path is empty".to_string()))
        }
    }

    pub fn curve3(&mut self, p0: Point, p1: Point, p2: Point) -> Result<()> {
        if let Some(last) = self.subpaths.last_mut() {
            last.add_segment(PathSegment::Curve3(BezierQuad::new(p0, p1, p2)));
            Ok(())
        } else {
            Err(PdfError::Path("Curve3 Path is empty".to_string()))
        }
    }

    pub fn rect(&mut self, rect: Rect) {
        let lf = rect.lower_left().to_owned();
        let mut sp = SubPath::new(rect.lower_left().to_owned());
        sp.add_segment(PathSegment::LineTo(Point::new(
            lf.x() + rect.width(),
            lf.y(),
        )));
        sp.add_segment(PathSegment::LineTo(Point::new(
            lf.x() + rect.width(),
            lf.y() + rect.height(),
        )));
        sp.add_segment(PathSegment::LineTo(Point::new(
            lf.x(),
            lf.y() + rect.height(),
        )));
        sp.close();
        self.subpaths.push(sp)
    }

    pub fn last_start_point(&self) -> Option<&Point> {
        match self.subpaths.first() {
            Some(sp) => sp.start_point(),
            None => None,
        }
    }

    pub fn close_sub_path(&mut self) -> Result<()> {
        if let Some(last) = self.subpaths.last_mut() {
            last.close();
        } else {
            return Err(PdfError::Path("Curve Path is empty".to_string()));
        }
        Ok(())
    }
    pub fn close_all(&mut self) -> Result<()> {
        for sub in self.subpaths.iter_mut() {
            sub.close()
        }
        Ok(())
    }
}
