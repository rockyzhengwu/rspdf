use crate::geom::point::Point;

#[derive(Debug, Default, Clone, Copy)]
pub struct Rectangle {
    lower_left: Point,
    uper_right: Point,
}

impl Rectangle {
    pub fn new(lx: f64, ly: f64, ux: f64, uy: f64) -> Self {
        let lower_left = Point::new(lx, ly);
        let uper_right = Point::new(ux, uy);
        Rectangle {
            lower_left,
            uper_right,
        }
    }

    pub fn lower_left(&self) -> &Point {
        &self.lower_left
    }

    pub fn lx(&self) -> f64 {
        self.lower_left.x()
    }
    pub fn ly(&self) -> f64 {
        self.lower_left.y()
    }

    pub fn ux(&self) -> f64 {
        self.uper_right.x()
    }

    pub fn uy(&self) -> f64 {
        self.uper_right.y()
    }

    pub fn width(&self) -> f64 {
        (self.ux() - self.lx()).abs()
    }
    pub fn height(&self) -> f64 {
        (self.uy() - self.ly()).abs()
    }

    pub fn merge(&mut self, other: &Rectangle) {
        let lx = self.lx().min(other.lx());
        let ly = self.ly().min(other.ly());
        let ux = self.ux().max(other.ux());
        let uy = self.uy().max(other.uy());
        self.lower_left = Point::new(lx, ly);
        self.uper_right = Point::new(ux, uy);
    }
}
