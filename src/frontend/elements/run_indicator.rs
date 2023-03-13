use macroquad::prelude::*;

use crate::{
    common::coord::{HexCoord, Point},
    core::game::UiAction,
    frontend::{
        element::{Element, ShapeState},
        events::{Event, Message},
        mouse::mouse_leave_enter_event,
    },
};

pub struct RunIndicator {
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

impl RunIndicator {
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

impl Element for RunIndicator {
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
