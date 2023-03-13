use crate::{
    common::coord::{HexCoord, Point},
    core::{entities::Player, game::UiAction},
    frontend::{
        animation::{Animation, FlipAnimation},
        element::{Element, ShapeState},
        events::{Message, Event},
    },
};
use macroquad::prelude::*;

use super::token::Token;

pub struct AnimatedToken {
    token: Token,
    animation: Option<Box<dyn Animation>>,
}

impl AnimatedToken {
    pub fn new(token: Token, animation: Box<dyn Animation>) -> Self {
        AnimatedToken {
            token,
            animation: Some(animation),
        }
    }

    pub fn ring(
        player: Player,
        coord: HexCoord,
        z_value: i32,
        animation: Box<dyn Animation>,
    ) -> Self {
        let ring = Token::new_ring_at_coord(coord, player, z_value);
        Self::new(ring, animation)
    }

    pub fn marker(
        player: Player,
        coord: HexCoord,
        z_value: i32,
        animation: Box<dyn Animation>,
    ) -> Self {
        let marker = Token::new_marker_at_coord(coord, player, z_value);
        Self::new(marker, animation)
    }
}

impl AnimatedToken {
    pub fn pos(&self) -> Point {
        self.token.pos()
    }

    pub fn coord(&self) -> Option<HexCoord> {
        self.token.coord()
    }

    pub fn set_pos(&mut self, pos: Point) {
        self.token.set_pos(pos);
    }

    pub fn contains(&self, pos: Point) -> bool {
        self.token.contains(pos)
    }
}

impl Element for AnimatedToken {
    fn render(&self) {
        self.token.render();
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        let mut res = self.token.update(message);
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
                animation.apply(&mut self.token);
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
        let mut res = self.token.handle_event(event);

        match event {
            Event::FlipMarker(coord) => {
                if Some(*coord) == self.token.coord() {
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
        self.token.set_state(state);
    }

    fn z_value(&self) -> i32 {
        self.token.z_value()
    }
}
