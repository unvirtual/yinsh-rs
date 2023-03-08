use crate::common::coord::*;
use crate::core::entities::{Piece, Player};
use crate::core::game::UserAction;
use crate::frontend::primitives::*;
use macroquad::audio::PlaySoundParams;
use macroquad::prelude::*;

use super::mcview::ShapeId;
use super::mouse::{mouse_leave_enter_event, MouseEvent};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ShapeState {
    Visible,
    Invisible,
    Selected,
    AtMousePointer,
    Hoverable,
}

pub trait Element {
    fn render(&self);
    fn update(&mut self, message: &Message) -> Option<UserAction>;
    fn handle_events(&self, mouse_event: &MouseEvent) -> Vec<Message>;

    fn pos(&self) -> Point;
    fn coord(&self) -> Option<HexCoord>;
    fn set_state(&mut self, state: ShapeState);
    fn set_pos(&mut self, pos: Point);
    fn contains(&self, pos: Point) -> bool;
    fn z_value(&self) -> i32;
}

fn player_color(player: Player) -> Color {
    match player {
        Player::White => WHITE,
        Player::Black => BLACK,
    }
}

pub enum ElementType {
    Ring(f32, f32),
    Marker(f32),
}

pub struct PieceElement {
    pos: Point,
    coord: Option<HexCoord>,
    shape_type: ElementType,
    color: Color,
    default_color: Color,
    hover_color: Color,
    state: ShapeState,
    z_value: i32,
}

impl PieceElement {
    pub fn new(
        pos: Point,
        coord: Option<HexCoord>,
        shape_type: ElementType,
        color: Color,
        z_value: i32,
    ) -> Self {
        PieceElement {
            pos,
            coord,
            shape_type,
            color,
            default_color: color,
            hover_color: BLUE,
            state: ShapeState::Visible,
            z_value,
        }
    }

    pub fn new_marker_at_coord(coord: HexCoord, player: Player, z_value: i32) -> Self {
        let pos = Point::from(coord);
        PieceElement::new(
            pos,
            Some(coord),
            ElementType::Marker(0.2),
            player_color(player),
            z_value,
        )
    }

    pub fn new_marker_at_point(pos: Point, player: Player, z_value: i32) -> Self {
        PieceElement::new(
            pos,
            None,
            ElementType::Marker(0.2),
            player_color(player),
            z_value,
        )
    }

    pub fn new_ring_at_coord(coord: HexCoord, player: Player, z_value: i32) -> Self {
        let pos = Point::from(coord);
        PieceElement::new(
            pos,
            Some(coord),
            ElementType::Ring(0.4, 0.2),
            player_color(player),
            z_value,
        )
    }

    pub fn new_ring_at_point(pos: Point, player: Player, z_value: i32) -> Self {
        PieceElement::new(
            pos,
            None,
            ElementType::Ring(0.4, 0.2),
            player_color(player),
            z_value,
        )
    }

    fn draw(&self, color: Color) {
        match self.shape_type {
            ElementType::Ring(radius_outer, radius_inner) => {
                draw_circle_lines(self.pos.0, self.pos.1, radius_outer, 0.03, BLACK);
                draw_circle_lines(self.pos.0, self.pos.1, radius_inner, 0.03, BLACK);
                draw_ring_mesh(self.pos.0, self.pos.1, radius_inner, radius_outer, color);
            }
            ElementType::Marker(radius) => {
                draw_circle(self.pos.0, self.pos.1, radius, color);
                draw_circle_lines(self.pos.0, self.pos.1, radius, 0.03, BLACK);
            }
        }
    }
}

impl Element for PieceElement {
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

    fn update(&mut self, event: &Message) -> Option<UserAction> {
        println!("REceived message: {:?}", event);
        match event {
            Message::MouseEntered => self.color = self.hover_color,
            Message::MouseLeft => self.color = self.default_color,
            Message::ElementMoved(pt) => self.pos = *pt,
            _ => (),
        }
        None
    }

    fn contains(&self, pos: Point) -> bool {
        match self.shape_type {
            ElementType::Marker(radius) => distance_squared(&self.pos, &pos) <= radius.powi(2),
            ElementType::Ring(outer, _) => distance_squared(&self.pos, &pos) <= outer.powi(2),
        }
    }

    fn handle_events(&self, mouse_event: &MouseEvent) -> Vec<Message> {
        let mut res = vec![];
        if self.state == ShapeState::Hoverable {
            mouse_leave_enter_event(mouse_event, |pt| self.contains(*pt)).map(|e| {
                res.push(e);
            });
        }
        if self.state == ShapeState::AtMousePointer {
            let pos = mouse_event
                .legal_move_coord
                .map(Point::from)
                .unwrap_or(mouse_event.pos);
            res.push(Message::ElementMoved(pos));
        }
        res
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}

pub struct FieldMarker {
    pos: Point,
    z_value: i32,
    radius: f32,
    mouse_radius: f32,
    coord: HexCoord,
}

impl FieldMarker {
    pub fn new(coord: HexCoord, radius: f32, mouse_radius: f32, z_value: i32) -> Self {
        Self {
            pos: Point::from(coord),
            coord,
            radius,
            mouse_radius,
            z_value,
        }
    }
}

impl Element for FieldMarker {
    fn render(&self) {
        draw_circle(self.pos.0, self.pos.1, self.radius, BLUE);
    }

    fn update(&mut self, message: &Message) -> Option<UserAction> {
        match message {
            Message::MouseClicked(_) => Some(UserAction::ActionAtCoord(self.coord)),
            _ => None,
        }
    }

    fn handle_events(&self, mouse_event: &MouseEvent) -> Vec<Message> {
        let mut res = vec![];
        if mouse_event.left_clicked && self.contains(mouse_event.pos) {
            res.push(Message::MouseClicked(self.coord));
        }
        res
    }

    fn pos(&self) -> Point {
        self.pos
    }

    fn coord(&self) -> Option<HexCoord> {
        Some(self.coord)
    }

    fn set_state(&mut self, state: ShapeState) {}

    fn set_pos(&mut self, pos: Point) {
        self.pos = pos
    }

    fn contains(&self, pos: Point) -> bool {
        distance_squared(&self.pos, &pos) <= self.mouse_radius.powi(2)
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}

pub struct RingMoveLineElement {
    pos: Point,
    target: Point,
    state: ShapeState,
    z_value: i32,
}

impl RingMoveLineElement {
    pub fn new(pos: Point, target: Point, z_value: i32) -> Self {
        Self {
            pos,
            target,
            state: ShapeState::Invisible,
            z_value,
        }
    }
}

impl Element for RingMoveLineElement {
    fn render(&self) {
        if self.state != ShapeState::Invisible {
            draw_line(
                self.pos.0,
                self.pos.1,
                self.target.0,
                self.target.1,
                0.1,
                BLUE,
            );
        }
    }

    fn update(&mut self, message: &Message) -> Option<UserAction> {
        match message {
            Message::ElementMoved(pos) => self.target = *pos,
            Message::ElementShow => self.state = ShapeState::Visible,
            Message::ElementHide => self.state = ShapeState::Invisible,
            _ => (),
        }
        None
    }

    fn handle_events(&self, mouse_event: &MouseEvent) -> Vec<Message> {
        let mut res = vec![];
        if let Some(pos) = mouse_event.legal_move_coord.map(Point::from) {
            if self.state == ShapeState::Invisible {
                res.push(Message::ElementShow);
            }
            res.push(Message::ElementMoved(pos));
        } else {
            if self.state == ShapeState::Visible {
                res.push(Message::ElementHide);
            }
        }

        res
    }

    fn pos(&self) -> Point {
        self.pos
    }

    fn coord(&self) -> Option<HexCoord> {
        None
    }

    fn set_state(&mut self, state: ShapeState) {
        self.state = state;
    }

    fn set_pos(&mut self, pos: Point) {
        self.pos = pos
    }

    fn contains(&self, pos: Point) -> bool {
        false
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}

pub struct RunBBoxElement {
    corners: [Vec2; 4],
    dir: Vec2,
    perp: Vec2,
    coord: Option<HexCoord>,
    width: f32,
    height: f32,
    z_value: i32,
    color: Color,
    value: Option<HexCoord>,
}

impl RunBBoxElement {
    pub fn new(corners: [Point; 4]) -> Self {
        let corners = corners.map(|v| Vec2::new(v.0, v.1));
        let dir = (corners[1] - corners[0]).normalize();
        let perp = Vec2::new(dir.y, -dir.x);
        let width = (corners[1] - corners[2]).length();
        let height = (corners[0] - corners[1]).length();

        Self {
            corners,
            z_value: 3,
            dir,
            perp,
            color: BLACK,
            width,
            height,
            coord: None,
            value: None,
        }
    }

    pub fn from_segment_coords(coord0: HexCoord, coord1: HexCoord, height: f32) -> Self {
        Self::from_segment_points(coord0.into(), coord1.into(), height)
    }

    pub fn set_coord(&mut self, coord: HexCoord) {
        self.coord = Some(coord);
    }

    pub fn from_segment_points(pt0: Point, pt1: Point, height: f32) -> Self {
        let v1 = Vec2::from((pt0.0, pt0.1));
        let v2 = Vec2::from((pt1.0, pt1.1));
        let dir = (v2 - v1).normalize();
        let perp = Vec2::new(dir.y, -dir.x);
        let width = (v2 - v1).length();

        let corners = [
            v1 + height / 2. * perp,
            v1 - height / 2. * perp,
            v2 - height / 2. * perp,
            v2 + height / 2. * perp,
        ];

        Self {
            corners,
            z_value: 3,
            dir,
            perp,
            color: BLACK,
            width,
            height,
            coord: None,
            value: None,
        }
    }
}

impl Element for RunBBoxElement {
    fn render(&self) {
        let thickness = 0.05;
        for i in 0..4 {
            draw_line(
                self.corners[i].x,
                self.corners[i].y,
                self.corners[(i + 1) % 4].x,
                self.corners[(i + 1) % 4].y,
                thickness,
                self.color,
            );
        }
    }

    fn update(&mut self, message: &Message) -> Option<UserAction> {
        match message {
            Message::MouseEntered => {
                self.color = GREEN;
                None
            }
            Message::MouseLeft => {
                self.color = BLACK;
                None
            }
            Message::MouseClicked(_) => self.coord.map(|c| UserAction::ActionAtCoord(c)),
            _ => None,
        }
    }

    fn handle_events(&self, mouse_event: &MouseEvent) -> Vec<Message> {
        let mut res = vec![];
        mouse_leave_enter_event(mouse_event, |pt| self.contains(*pt)).map(|e| {
            res.push(e);
        });
        if mouse_event.left_clicked && self.contains(mouse_event.pos) && self.coord.is_some() {
            res.push(Message::MouseClicked(self.coord.unwrap()));
        }
        res
    }

    fn pos(&self) -> Point {
        Point(self.corners[0].x, self.corners[0].y)
    }

    fn coord(&self) -> Option<HexCoord> {
        None
    }

    fn set_state(&mut self, state: ShapeState) {}

    fn set_pos(&mut self, pos: Point) {}

    fn contains(&self, pos: Point) -> bool {
        let height = (self.corners[0] - self.corners[1]).length();
        let start = self.corners[0] - self.perp * height / 2.;
        let pt = vec2(pos.0, pos.1);

        let diff = pt - start;

        let proj = diff.dot(self.dir);
        if proj < 0. || proj > (self.corners[1] - self.corners[2]).length() {
            return false;
        }

        (diff - proj * self.dir).length_squared() <= (height / 2.).powi(2)
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}
