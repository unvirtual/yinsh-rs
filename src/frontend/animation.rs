use std::f32::consts::PI;

use crate::common::coord::Point;
use macroquad::prelude::*;

use super::elements::token::{Token, TokenType};

pub trait Animation {
    fn tick(&mut self);
    fn finished(&self) -> bool;
    fn apply(&self, marker: &mut Token);
}

#[derive(Clone)]
pub struct FlipAnimation {
    start_time: f64,
    duration: f64,
    start_color: Color,
    end_color: Color,
    current_color: Color,
}

impl FlipAnimation {
    pub fn new(start_color: Color, end_color: Color) -> Self {
        FlipAnimation {
            start_time: get_time(),
            duration: 0.2,
            start_color,
            end_color,
            current_color: start_color,
        }
    }

    pub fn new_box(start_color: Color, end_color: Color) -> Box<Self> {
        Box::new(Self::new(start_color, end_color))
    }
}

impl Animation for FlipAnimation {
    fn tick(&mut self) {
        let delta = (1. / self.duration * (get_time() - self.start_time)) as f32;
        self.current_color = Color::from_vec(
            self.start_color.to_vec()
                + delta * (self.end_color.to_vec() - self.start_color.to_vec()),
        );
    }

    fn apply(&self, marker: &mut Token) {
        marker.set_color(self.current_color);
    }

    fn finished(&self) -> bool {
        get_time() - self.start_time > self.duration
    }
}

#[derive(Clone)]
pub struct RemoveAnimation {
    start_time: f64,
    duration: f64,
    amplitude: f32,
    phase_shift: f32,
    value: f32,
}

impl RemoveAnimation {
    pub fn new(expand_ratio: f32) -> Self {
        let phase_shift = (1. / expand_ratio).asin();

        RemoveAnimation {
            start_time: get_time(),
            duration: 0.2,
            phase_shift,
            amplitude: expand_ratio,
            value: 1.,
        }
    }

    pub fn new_box(expand_ratio: f32) -> Box<Self> {
        Box::new(Self::new(expand_ratio))
    }
}

impl Animation for RemoveAnimation {
    fn tick(&mut self) {
        let t = (1. / self.duration * (get_time() - self.start_time)) as f32;
        let delta = self.phase_shift + t * (PI - self.phase_shift);
        self.value = self.amplitude * delta.sin();
    }

    fn apply(&self, marker: &mut Token) {
        match marker.shape_type {
            TokenType::Ring(r1, r2) => {
                marker.shape_type = TokenType::Ring(self.value * r1, self.value * r2)
            }
            TokenType::Marker(r) => marker.shape_type = TokenType::Marker(self.value * r),
        }
    }

    fn finished(&self) -> bool {
        get_time() - self.start_time > self.duration
    }
}

#[derive(Clone)]
pub struct MoveAnimation {
    start_time: f64,
    duration: f64,
    start_pos: Point,
    end_pos: Point,
    current_pos: Point,
}

impl MoveAnimation {
    pub fn new(start_pos: Point, end_pos: Point) -> Self {
        MoveAnimation {
            start_time: get_time(),
            duration: 0.5,
            start_pos,
            end_pos,
            current_pos: start_pos,
        }
    }

    pub fn new_box(start_pos: Point, end_pos: Point) -> Box<Self> {
        Box::new(Self::new(start_pos, end_pos))
    }
}

impl Animation for MoveAnimation {
    fn tick(&mut self) {
        if self.finished() {
            self.current_pos = self.end_pos;
        } else {
            let delta = (1. / self.duration * (get_time() - self.start_time)) as f32;
            self.current_pos = self.start_pos + (self.end_pos - self.start_pos) * delta;
        }
    }

    fn apply(&self, ring: &mut Token) {
        ring.set_pos(self.current_pos);
    }

    fn finished(&self) -> bool {
        get_time() - self.start_time >= self.duration
    }
}
