use macroquad::prelude::*;
use crate::common::coord::*;
use crate::frontend::primitives::*;

pub trait Shape {
    fn render(&self);
    fn pos(&self) -> Point;
    fn coord(&self) -> HexCoord;
}

pub struct MarkerShape {
    pos: Point,
    coord: HexCoord,
    radius: f32,
    color: Color,
}

impl MarkerShape {
    pub fn new(pos: Point, coord: HexCoord, radius: f32, color: Color) -> Self {
        MarkerShape { pos, coord, radius, color }
    }

    pub fn white(coord: HexCoord) -> Self {
        let pos = Point::from(coord);
        MarkerShape::new(pos, coord, 0.2, WHITE)
    }

    pub fn black(coord: HexCoord) -> Self {
        let pos = Point::from(coord);
        MarkerShape::new(pos, coord, 0.2, BLACK)
    }
}

impl Shape for MarkerShape {
    fn render(&self) {
        draw_circle(self.pos.0, self.pos.1, self.radius, self.color);
    }

    fn pos(&self) -> Point {
        self.pos
    }

    fn coord(&self) -> HexCoord {
        self.coord
    }
}

pub struct RingShape {
    pos: Point,
    coord: HexCoord,
    radius_outer: f32,
    radius_inner: f32,
    color: Color,
}

impl RingShape {
    pub fn new(pos: Point, coord: HexCoord, radius_outer: f32, radius_inner: f32, color: Color) -> Self {
        RingShape {
            pos,
            coord,
            radius_outer,
            radius_inner,
            color,
        }
    }

    pub fn white(coord: HexCoord) -> Self {
        let pos = Point::from(coord);
        RingShape::new(pos, coord, 0.4, 0.2, WHITE)
    }

    pub fn black(coord: HexCoord) -> Self {
        let pos = Point::from(coord);
        RingShape::new(pos, coord, 0.4, 0.2, BLACK)
    }
}

impl Shape for RingShape {
    fn render(&self) {
        draw_ring_mesh(
            self.pos.0,
            self.pos.1,
            self.radius_inner,
            self.radius_outer,
            self.color,
        );
    }

    fn pos(&self) -> Point {
        self.pos
    }

    fn coord(&self) -> HexCoord {
        self.coord
    }
}
