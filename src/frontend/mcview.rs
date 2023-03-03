use crate::common::coord::*;
use crate::core::board::*;
use crate::core::entities::*;
use crate::core::game::*;
use crate::core::state::*;
use macroquad::prelude::*;

use super::shape::*;

pub struct MCFrontend {
    grid_lines: Vec<[HexCoordF; 2]>,
    width: f32,
    height: f32,
    pixel_width: u32,
    pixel_height: u32,
    shapes: Vec<Box<dyn Shape>>,
    pointed_shape_idx: Option<usize>,
}

impl View for MCFrontend {
    fn update(&mut self, state: &State) {
        self.shapes.clear();
        state
            .board
            .player_rings(Player::White)
            .for_each(|c| self.shapes.push(Box::new(RingShape::white(*c))));
        state
            .board
            .player_rings(Player::Black)
            .for_each(|c| self.shapes.push(Box::new(RingShape::black(*c))));
        state
            .board
            .player_markers(Player::White)
            .for_each(|c| self.shapes.push(Box::new(MarkerShape::white(*c))));
        state
            .board
            .player_markers(Player::Black)
            .for_each(|c| self.shapes.push(Box::new(MarkerShape::black(*c))));
    }

    fn render(&self) {
        clear_background(LIGHTGRAY);
        self.set_camera();
        self.draw_grid();
        self.shapes.iter().for_each(|s| s.render());
    }

    fn poll_user_action(&self) -> Option<UserAction> {
        if is_mouse_button_pressed(MouseButton::Left) {
            println!("Left MButton pressed");
            if let Some(coord) = self.mouse_position_to_coord(Some(0.04)) {
                println!("Close to coord {:?}", coord);
                return Some(UserAction::ActionAtCoord(coord));
            }
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            return Some(UserAction::Undo);
        }

        None
    }

    fn invalid_action(&self) {
        println!("MCFrontend: Invalid action");
    }
}

impl MCFrontend {
    fn build_grid_lines(radius: f32) -> Vec<[HexCoordF; 2]> {
        let dx: f32 = 0.5 * (3. as f32).sqrt();
        let mut res = Vec::new();

        // diagonals
        for dy in [-0.5 as f32, 0.5 as f32] {
            let lambda: f32 = radius / (1. - dy.powi(2)).sqrt();
            let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

            for l in l0..=l1 {
                let l = l as f32;
                let det = (l.powi(2) * (dy.powi(2) - 1.) + radius.powi(2)).sqrt();
                if det <= 0. {
                    continue;
                }
                let mut mu1 = -l * dy - det;
                let mut mu2 = -l * dy + det;
                if l.abs() > radius {
                    mu1 = mu1.ceil();
                    mu2 = mu2.floor();
                } else {
                    mu1 = mu1.trunc();
                    mu2 = mu2.floor();
                }

                let vec = [
                    HexCoordF(mu1 * dx, l + mu1 * dy),
                    HexCoordF(mu2 * dx, l + mu2 * dy),
                ];
                res.push(vec);
            }
        }

        // verticals
        let lambda: f32 = radius * 2. / 3. * (3. as f32).sqrt();
        let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

        for l in l0..=l1 {
            let l = l as f32;
            let det = (4. * radius.powi(2) - 3. * l.powi(2)).sqrt();
            if det <= 0. {
                continue;
            }
            let mut mu1 = 0.5 * (l - det);
            let mut mu2 = 0.5 * (l + det);
            if l.abs() > radius {
                mu1 = mu1.ceil();
                mu2 = mu2.floor();
            } else {
                mu1 = mu1.trunc();
                mu2 = mu2.floor();
            }

            let vec = [
                HexCoordF(l * dx, -0.5 * l + mu1),
                HexCoordF(l * dx, -0.5 * l + mu2),
            ];
            res.push(vec);
        }
        res
    }

    pub fn new(
        board: &Board,
        pixel_width: u32,
        pixel_height: u32,
        w_margin: f32,
        h_margin: f32,
    ) -> Self {
        let radius = board.get_radius();

        MCFrontend {
            grid_lines: Self::build_grid_lines(radius),
            width: (2. * radius + w_margin),
            height: (2. * radius + h_margin),
            pixel_width,
            pixel_height,
            shapes: vec![],
            pointed_shape_idx: None,
        }
    }

    fn pixels_to_xy(&self, px: f32, py: f32) -> (f32, f32) {
        let h_ratio = self.pixel_width as f32 / self.width;
        let w_ratio = self.pixel_height as f32 / self.height;

        (
            1. / w_ratio * (px - self.pixel_width as f32 / 2.),
            1. / h_ratio * (py - self.pixel_height as f32 / 2.),
        )
    }

    fn mouse_position_to_coord(&self, max_sq_dist: Option<f32>) -> Option<HexCoord> {
        let maxd = max_sq_dist.unwrap_or(f32::INFINITY);

        let (px, py) = mouse_position();
        let (x, y) = self.pixels_to_xy(px, py);
        let (coord, sq_dist) = HexCoord::closest_coord_to_point(&Point(x, -y));

        if sq_dist <= maxd {
            Some(coord)
        } else {
            None
        }
    }

    fn set_camera(&self) {
        set_camera(&Camera2D {
            zoom: vec2(1. / self.width * 2., 1. / self.height * 2.),
            target: vec2(0., 0.),
            //rotation: 179.,
            ..Default::default()
        });
    }

    fn draw_grid(&self) {
        for [p0, p1] in &self.grid_lines {
            // let p0 = self.to_pixel(p0.0, p0.1);
            // let p1 = self.to_pixel(p1.0, p1.1);
            draw_line(p0.0, p0.1, p1.0, p1.1, 0.02, DARKGRAY);
        }
    }
}
