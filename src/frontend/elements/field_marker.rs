use macroquad::prelude::*;

use crate::{
    common::coord::{distance_squared, HexCoord, Point},
    core::game::UiAction,
    frontend::{
        element::{Element, ShapeState},
        events::{Event, Message},
    },
};

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
