use std::f32::consts::PI;

use crate::common::coord::*;
use crate::core::entities::{Piece, Player};
use crate::core::game::UiAction;
use crate::frontend::primitives::*;
use macroquad::audio::PlaySoundParams;
use macroquad::prelude::*;

use super::animation::*;
use super::events::*;
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
