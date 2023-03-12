use std::f32::consts::PI;

use crate::common::coord::*;
use crate::core::entities::{Piece, Player};
use crate::core::game::UiAction;
use crate::frontend::primitives::*;
use macroquad::audio::PlaySoundParams;
use macroquad::prelude::*;

use super::animation::*;
use super::frontend::ShapeId;
use super::mouse::{mouse_leave_enter_event, MouseEvent};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ShapeState {
    Visible,
    Invisible,
    Selected,
    AtMousePointer,
    Hoverable,
    Animated,
}

pub trait Element {
    fn render(&self);
    fn update(&mut self, message: &Message) -> Option<UiAction>;
    fn handle_event(&self, event: &Event) -> Vec<Message>;
    fn set_state(&mut self, state: ShapeState);
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

pub struct AnimatedPieceElement {
    element: PieceElement,
    animation: Option<Box<dyn Animation>>,
}

impl AnimatedPieceElement {
    pub fn new(piece: PieceElement, animation: Box<dyn Animation>) -> Self {
        AnimatedPieceElement {
            element: piece,
            animation: Some(animation),
        }
    }

    pub fn ring(
        player: Player,
        coord: HexCoord,
        z_value: i32,
        animation: Box<dyn Animation>,
    ) -> Self {
        let ring = PieceElement::new_ring_at_coord(coord, player, z_value);
        Self::new(ring, animation)
    }

    pub fn marker(
        player: Player,
        coord: HexCoord,
        z_value: i32,
        animation: Box<dyn Animation>,
    ) -> Self {
        let marker = PieceElement::new_marker_at_coord(coord, player, z_value);
        Self::new(marker, animation)
    }
}

impl AnimatedPieceElement {
    pub fn pos(&self) -> Point {
        self.element.pos()
    }

    pub fn coord(&self) -> Option<HexCoord> {
        self.element.coord()
    }

    pub fn set_pos(&mut self, pos: Point) {
        self.element.set_pos(pos);
    }

    pub fn contains(&self, pos: Point) -> bool {
        self.element.contains(pos)
    }
}

impl Element for AnimatedPieceElement {
    fn render(&self) {
        self.element.render();
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        let mut res = self.element.update(message);
        if res.is_some() {
            return res;
        }

        match message {
            Message::FlipMarker(_) => {
                let animation = FlipAnimation::new(WHITE, RED);
                return Some(UiAction::AnimationInProgress);
            }
            Message::Tick => {
                if self.animation.is_none() {
                    return Some(UiAction::AnimationFinished);
                }
                let animation = self.animation.as_mut().unwrap();

                animation.tick();
                animation.apply(&mut self.element);
                if self.animation.as_ref().unwrap().finished() {
                    return Some(UiAction::AnimationFinished);
                } else {
                    return Some(UiAction::AnimationInProgress);
                }
            }
            _ => (),
        }
        res
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = self.element.handle_event(event);

        match event {
            Event::FlipMarker(coord) => {
                if Some(*coord) == self.element.coord {
                    res.push(Message::FlipMarker(*coord));
                }
            }
            _ => (),
        }
        if self.animation.is_some() {
            res.push(Message::Tick);
        }
        res
    }

    fn set_state(&mut self, state: ShapeState) {
        self.element.set_state(state);
    }

    fn z_value(&self) -> i32 {
        self.element.z_value()
    }
}

pub struct PieceElement {
    pos: Point,
    coord: Option<HexCoord>,
    pub shape_type: ElementType,
    color: Color,
    default_color: Color,
    hover_color: Color,
    other_color: Color,
    state: ShapeState,
    z_value: i32,
    mouse_entered: bool,
}

impl PieceElement {
    pub fn new(
        pos: Point,
        coord: Option<HexCoord>,
        shape_type: ElementType,
        color: Color,
        other_color: Color,
        z_value: i32,
    ) -> Self {
        PieceElement {
            pos,
            coord,
            shape_type,
            color,
            default_color: color,
            other_color: other_color,
            hover_color: BLUE,
            state: ShapeState::Visible,
            z_value,
            mouse_entered: false,
        }
    }

    pub fn new_marker_at_coord(coord: HexCoord, player: Player, z_value: i32) -> Self {
        let pos = Point::from(coord);
        let mut elem = PieceElement::new_marker_at_point(pos, player, z_value);
        elem.coord = Some(coord);
        elem
    }

    pub fn new_marker_at_point(pos: Point, player: Player, z_value: i32) -> Self {
        PieceElement::new(
            pos,
            None,
            ElementType::Marker(0.2),
            player_color(player),
            player_color(player.other()),
            z_value,
        )
    }

    pub fn new_ring_at_coord(coord: HexCoord, player: Player, z_value: i32) -> Self {
        let pos = Point::from(coord);
        let mut elem = PieceElement::new_ring_at_point(pos, player, z_value);
        elem.coord = Some(coord);
        elem
    }

    pub fn new_ring_at_point(pos: Point, player: Player, z_value: i32) -> Self {
        PieceElement::new(
            pos,
            None,
            ElementType::Ring(0.4, 0.2),
            player_color(player),
            player_color(player.other()),
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

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn contains(&self, pos: Point) -> bool {
        match self.shape_type {
            ElementType::Marker(radius) => distance_squared(&self.pos, &pos) <= radius.powi(2),
            ElementType::Ring(outer, _) => distance_squared(&self.pos, &pos) <= outer.powi(2),
        }
    }

    fn pos(&self) -> Point {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Point) {
        self.pos = pos;
    }

    fn coord(&self) -> Option<HexCoord> {
        self.coord
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

    fn update(&mut self, event: &Message) -> Option<UiAction> {
        match event {
            Message::MouseEntered => {
                self.color = self.hover_color;
                self.mouse_entered = true;
            }
            Message::MouseLeft => self.color = self.default_color,
            Message::ElementMoved(pt) => self.pos = *pt,
            _ => (),
        }
        None
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if self.state == ShapeState::Hoverable {
                    if let Some(e) = mouse_leave_enter_event(mouse_event, |pt| self.contains(*pt)) {
                        res.push(e);
                        return res;
                    };
                    if self.contains(mouse_event.pos) {
                        res.push(Message::MouseInside);
                    }
                }
                if self.state == ShapeState::AtMousePointer {
                    let pos = mouse_event
                        .legal_move_coord
                        .map(Point::from)
                        .unwrap_or(mouse_event.pos);
                    res.push(Message::ElementMoved(pos));
                }
            }
            _ => (),
        }
        res
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

impl FieldMarker {
    fn pos(&self) -> Point {
        self.pos
    }

    fn coord(&self) -> Option<HexCoord> {
        Some(self.coord)
    }

    fn set_pos(&mut self, pos: Point) {
        self.pos = pos
    }

    fn contains(&self, pos: Point) -> bool {
        distance_squared(&self.pos, &pos) <= self.mouse_radius.powi(2)
    }
}

impl Element for FieldMarker {
    fn render(&self) {
        draw_circle(self.pos.0, self.pos.1, self.radius, BLUE);
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            Message::MouseClicked(_) => Some(UiAction::ActionAtCoord(self.coord)),
            _ => None,
        }
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if mouse_event.left_clicked && self.contains(mouse_event.pos) {
                    res.push(Message::MouseClicked(self.coord));
                }
            }
            _ => (),
        }
        res
    }
    fn set_state(&mut self, state: ShapeState) {}

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

impl RingMoveLineElement {
    fn pos(&self) -> Point {
        self.pos
    }

    fn set_pos(&mut self, pos: Point) {
        self.pos = pos
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

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            Message::ElementMoved(pos) => self.target = *pos,
            Message::ElementShow => self.state = ShapeState::Visible,
            Message::ElementHide => self.state = ShapeState::Invisible,
            _ => (),
        }
        None
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
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
            }
            _ => (),
        }

        res
    }

    fn set_state(&mut self, state: ShapeState) {
        self.state = state;
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
    mouse_entered: bool,
}

impl RunBBoxElement {
    pub fn new(corners: [Point; 4], z_value: i32) -> Self {
        let corners = corners.map(|v| Vec2::new(v.0, v.1));
        let dir = (corners[1] - corners[0]).normalize();
        let perp = Vec2::new(dir.y, -dir.x);
        let width = (corners[1] - corners[2]).length();
        let height = (corners[0] - corners[1]).length();

        Self {
            corners,
            z_value,
            dir,
            perp,
            color: BLACK,
            width,
            height,
            coord: None,
            value: None,
            mouse_entered: false,
        }
    }

    pub fn from_segment_coords(
        coord0: HexCoord,
        coord1: HexCoord,
        height: f32,
        z_value: i32,
    ) -> Self {
        Self::from_segment_points(coord0.into(), coord1.into(), height, z_value)
    }

    pub fn set_coord(&mut self, coord: HexCoord) {
        self.coord = Some(coord);
    }

    pub fn from_segment_points(pt0: Point, pt1: Point, height: f32, z_value: i32) -> Self {
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
            z_value,
            dir,
            perp,
            color: BLACK,
            width,
            height,
            coord: None,
            value: None,
            mouse_entered: false,
        }
    }

    fn pos(&self) -> Point {
        Point(self.corners[0].x, self.corners[0].y)
    }

    fn coord(&self) -> Option<HexCoord> {
        None
    }

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

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            Message::MouseEntered => {
                self.color = GREEN;
                self.mouse_entered = true;
                None
            }
            Message::MouseLeft => {
                self.color = BLACK;
                None
            }
            Message::MouseClicked(_) => self.coord.map(|c| UiAction::ActionAtCoord(c)),
            _ => None,
        }
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if let Some(e) = mouse_leave_enter_event(mouse_event, |pt| self.contains(*pt)) {
                    res.push(e);
                    return res;
                };
                if self.contains(mouse_event.pos) {
                    res.push(Message::MouseInside);
                }
                if mouse_event.left_clicked
                    && self.contains(mouse_event.pos)
                    && self.coord.is_some()
                {
                    res.push(Message::MouseClicked(self.coord.unwrap()));
                }
            }
            _ => (),
        }
        res
    }

    fn set_state(&mut self, state: ShapeState) {}

    fn z_value(&self) -> i32 {
        self.z_value
    }
}
