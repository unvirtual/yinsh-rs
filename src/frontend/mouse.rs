use macroquad::prelude::{is_mouse_button_pressed, mouse_position, MouseButton};

use crate::core::command::Command;
use crate::{
    common::coord::{HexCoord, Point},
    core::actions::Action,
};

use super::events::Message;

#[derive(PartialEq, Clone, Debug)]
pub struct MouseEvent {
    pub pos: Point,
    pub last_pos: Point,
    pub coord: Option<HexCoord>,
    pub legal_move_coord: Option<HexCoord>,
    pub left_clicked: bool,
    pub right_clicked: bool,
}

pub fn mouse_leave_enter_event<T>(mouse_event: &MouseEvent, contains: T) -> Option<Message>
where
    T: Fn(&Point) -> bool,
{
    let mut msg = None;
    let from = &mouse_event.last_pos;
    let to = &mouse_event.pos;

    if !contains(from) && contains(to) {
        println!("mouse enter");
        msg = Some(Message::MouseEntered);
    }
    if contains(from) && !contains(to) {
        println!("mouse leave");
        msg = Some(Message::MouseLeft);
    }
    msg
}

pub struct MouseHandler {
    pos: Point,
    last_pos: Point,
    height: f32,
    width: f32,
    pixel_height: u32,
    pixel_width: u32,
}

impl MouseHandler {
    pub fn new(width: f32, height: f32, pixel_width: u32, pixel_height: u32) -> Self {
        Self {
            width,
            height,
            pixel_height,
            pixel_width,
            pos: Point(0., 0.),
            last_pos: Point(0., 0.),
        }
    }

    pub fn update(&mut self) {
        self.last_pos = self.pos;
        let mp = mouse_position();
        let (x, y) = self.pixels_to_xy(mp.0, mp.1);
        self.pos = Point(x, y);
    }

    pub fn has_message(&self, legal_moves: Option<&Vec<Action>>) -> MouseEvent {
        let left_clicked = is_mouse_button_pressed(MouseButton::Left);
        let right_clicked = is_mouse_button_pressed(MouseButton::Right);

        MouseEvent {
            last_pos: self.last_pos,
            pos: self.pos,
            coord: self.to_coord(Some(0.09)),
            legal_move_coord: legal_moves.and_then(|l| self.to_legal_field(l, Some(0.09))),
            left_clicked,
            right_clicked,
        }
    }

    pub fn to_coord(&self, max_sq_dist: Option<f32>) -> Option<HexCoord> {
        let maxd = max_sq_dist.unwrap_or(f32::INFINITY);

        let (coord, sq_dist) = HexCoord::closest_coord_to_point(&self.pos);
        if sq_dist <= maxd {
            Some(coord)
        } else {
            None
        }
    }

    pub fn to_legal_field(
        &self,
        legal_moves: &Vec<Action>,
        max_sq_dist: Option<f32>,
    ) -> Option<HexCoord> {
        let maxd = max_sq_dist.unwrap_or(f32::INFINITY);

        let (coord, sq_dist) = HexCoord::closest_coord_to_point(&self.pos);

        legal_moves
            .iter()
            .find(|&a| a.coord() == coord)
            .and(if sq_dist <= maxd { Some(coord) } else { None })
    }

    fn pixels_to_xy(&self, px: f32, py: f32) -> (f32, f32) {
        let h_ratio = self.pixel_width as f32 / self.width;
        let w_ratio = self.pixel_height as f32 / self.height;

        (
            1. / w_ratio * (px - self.pixel_width as f32 / 2.),
            -1. / h_ratio * (py - self.pixel_height as f32 / 2.),
        )
    }
}
