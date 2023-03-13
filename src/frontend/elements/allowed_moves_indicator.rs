use macroquad::prelude::*;

use crate::{
    common::coord::Point,
    core::game::UiAction,
    frontend::{
        element::{Element, ShapeState},
        events::{Event, Message},
    },
};

pub struct AllowedMovesIndicator {
    pos: Point,
    target: Point,
    state: ShapeState,
    z_value: i32,
}

impl AllowedMovesIndicator {
    pub fn new(pos: Point, target: Point, z_value: i32) -> Self {
        Self {
            pos,
            target,
            state: ShapeState::Invisible,
            z_value,
        }
    }

    fn pos(&self) -> Point {
        self.pos
    }

    fn set_pos(&mut self, pos: Point) {
        self.pos = pos
    }
}

impl Element for AllowedMovesIndicator {
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
