use crate::common::coord::*;
use crate::core::entities::{Player, Piece};
use crate::frontend::primitives::*;
use macroquad::audio::PlaySoundParams;
use macroquad::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ShapeState {
    Visible,
    Invisible,
    Selected,
    AtMousePointer,
    Hoverable,
}

#[derive(Eq, PartialEq)]
pub enum Event {
    MouseEntered,
    MouseLeft,
}

pub trait Shape {
    fn render(&self);
    fn update(&mut self, event: &Event);
    fn pos(&self) -> Point;
    fn coord(&self) -> Option<HexCoord>;
    fn set_state(&mut self, state: ShapeState);
    fn set_pos(&mut self, pos: Point);
}

fn player_color(player: Player) -> Color {
    match player {
        Player::White => WHITE,
        Player::Black => BLACK,
    }
}

pub enum ShapeType {
    Ring(f32, f32),
    Marker(f32),
}

pub struct PieceShape {
    pos: Point,
    coord: Option<HexCoord>,
    shape_type: ShapeType,
    color: Color,
    default_color: Color,
    hover_color: Color,
    state: ShapeState,
}

impl PieceShape {
    pub fn new(pos: Point, coord: Option<HexCoord>, shape_type: ShapeType, color: Color) -> Self {
        PieceShape {
            pos,
            coord,
            shape_type,
            color,
            default_color: color,
            hover_color: BLUE,
            state: ShapeState::Visible,
        }
    }

    pub fn new_marker_at_coord(coord: HexCoord, player: Player) -> Self {
        let pos = Point::from(coord);
        PieceShape::new(
            pos,
            Some(coord),
            ShapeType::Marker(0.2),
            player_color(player),
        )
    }

    pub fn new_marker_at_point(pos: Point, player: Player) -> Self {
        PieceShape::new(pos, None, ShapeType::Marker(0.2), player_color(player))
    }

    pub fn new_ring_at_coord(coord: HexCoord, player: Player) -> Self {
        let pos = Point::from(coord);
        PieceShape::new(
            pos,
            Some(coord),
            ShapeType::Ring(0.4, 0.2),
            player_color(player),
        )
    }

    pub fn new_ring_at_point(pos: Point, player: Player) -> Self {
        PieceShape::new(pos, None, ShapeType::Ring(0.4, 0.2), player_color(player))
    }

    fn draw(&self, color: Color) {
        match self.shape_type {
            ShapeType::Ring(radius_outer, radius_inner) => {
                draw_circle_lines(self.pos.0, self.pos.1, radius_outer, 0.03, BLACK);
                draw_circle_lines(self.pos.0, self.pos.1, radius_inner, 0.03, BLACK);
                draw_ring_mesh(
                    self.pos.0,
                    self.pos.1,
                    radius_inner,
                    radius_outer,
                    color,
                );
            }
            ShapeType::Marker(radius) => {
                draw_circle(self.pos.0, self.pos.1, radius, color);
                draw_circle_lines(self.pos.0, self.pos.1, radius, 0.03, BLACK);
            }
        }
    }
}

impl Shape for PieceShape {
    fn render(&self) {
        if self.state == ShapeState::Invisible {
            return;
        }
        if self.state == ShapeState::Selected {
            self.draw(BLUE);
        } else {
            self.draw(self.color);
        }
    }

    fn pos(&self) -> Point {
        self.pos
    }

    fn set_pos(&mut self, pos: Point) {
        self.pos = pos;
    }

    fn coord(&self) -> Option<HexCoord> {
        self.coord
    }

    fn set_state(&mut self, state: ShapeState) {
        self.state = state;
        match self.state {
            ShapeState::AtMousePointer => {
                self.color = Color::from_vec(self.color.to_vec() - vec4(0., 0., 0., 0.5));
            }
            _ => (),
        }
    }

    fn update(&mut self, event: &Event) {
        if self.state == ShapeState::Hoverable {
            self.color = match event {
                Event::MouseEntered => self.hover_color,
                Event::MouseLeft => self.default_color,
            }
        }
    }
}
