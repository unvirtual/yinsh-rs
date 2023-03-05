use crate::common::coord::*;
use crate::core::actions::*;
use crate::core::board::*;
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
    rings: Vec<Box<dyn Shape>>,
    markers: Vec<Box<dyn Shape>>,
    legal_moves: Vec<Action>,
    phase: Phase,
    current_player: Player,
    shape_at_mouse_pointer: Option<Box<dyn Shape>>,
    interactive: bool,
    groups: Vec<Box<dyn Shape>>,
}

impl View for MCFrontend {
    fn update(&mut self, state: &State) {
        self.current_player = state.current_player;
        self.phase = state.current_phase;
        self.update_shape_at_mouse_pointer(self.phase, self.current_player);

        self.legal_moves = state.legal_moves();

        self.rings.clear();
        self.markers.clear();

        for player in [Player::White, Player::Black] {
            state.board.player_rings(player).for_each(|c| {
                let mut s = Box::new(PieceShape::new_ring_at_coord(*c, player)); 
                if player == self.current_player && self.phase == Phase::RemoveRing {
                    s.set_state(ShapeState::Hoverable);
                }
                self.rings.push(s);
            });

            state.board.player_markers(player).for_each(|c| {
                let mut s = Box::new(PieceShape::new_marker_at_coord(*c, player));
                if player == self.current_player && self.phase == Phase::RemoveRun {

                    let runs = if player == Player::Black { 
                        &state.runs_black
                    } else {
                        &state.runs_white
                    };

                    if runs.iter().flatten().find(|rc| rc == &c).is_some() {
                        s.set_state(ShapeState::Selected);
                    }

                }
                self.markers.push(s);
            });
        }

    }

    fn render(&mut self) {
        clear_background(LIGHTGRAY);
        self.set_camera();
        self.draw_grid();

        self.legal_moves.iter().for_each(|a| {
            let pt: Point = a.coord().into();
            draw_circle(pt.0, pt.1, 0.1, BLUE);
        });

        // draw below objects and set properties before drawing
        if self.interactive {
            self.interactive_highlight(self.phase);
        }

        let mouse_pos = self.mouse_position_to_point();

        self.rings.iter_mut().for_each(|s| s.render());
        self.markers.iter_mut().for_each(|s| s.render());

        if !self.interactive {
            return;
        }

        let mouse_pos = self.mouse_pos_snapping_to_possible_moves();
        self.shape_at_mouse_pointer.as_mut().map(|s| {
            s.set_pos(mouse_pos);
            s.render();
        });
    }

    fn poll_user_action(&self) -> Option<UserAction> {
        if is_mouse_button_pressed(MouseButton::Left) {
            println!("Left MButton pressed");
            if let Some(coord) = self.mouse_position_to_coord(Some(0.09)) {
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

    fn set_interactive(&mut self, flag: bool) {
        self.interactive = flag;
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
            rings: vec![],
            markers: vec![],
            legal_moves: vec![],
            phase: Phase::PlaceRing,
            interactive: false,
            current_player: Player::White,
            shape_at_mouse_pointer: None,
            groups: vec![]
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

    fn mouse_position_to_legal_field(&self, max_sq_dist: Option<f32>) -> Option<HexCoord> {
        let maxd = max_sq_dist.unwrap_or(f32::INFINITY);

        let (px, py) = mouse_position();
        let (x, y) = self.pixels_to_xy(px, py);
        let (coord, sq_dist) = HexCoord::closest_coord_to_point(&Point(x, -y));

        self.legal_moves
            .iter()
            .find(|a| a.coord() == coord)
            .and(if sq_dist <= maxd { Some(coord) } else { None })
    }

    fn mouse_position_to_point(&self) -> Point {
        let (px, py) = mouse_position();
        let (x, y) = self.pixels_to_xy(px, py);
        Point(x, -y)
    }

    fn set_camera(&self) {
        set_camera(&Camera2D {
            zoom: vec2(1. / self.width * 2., 1. / self.height * 2.),
            target: vec2(0., 0.),
            ..Default::default()
        });
    }

    fn draw_grid(&self) {
        for [p0, p1] in &self.grid_lines {
            draw_line(p0.0, p0.1, p1.0, p1.1, 0.02, DARKGRAY);
        }
    }

    fn clear_shape_at_mouse_pointer(&mut self) {
        self.shape_at_mouse_pointer = None;
    }

    fn mouse_pos_snapping_to_possible_moves(&self) -> Point {
        let mut pt = self.mouse_position_to_point();
        if let Some(c) = self.mouse_position_to_legal_field(Some(0.09)) {
            pt = c.into();
        }
        pt
    }

    fn update_shape_at_mouse_pointer(&mut self, phase: Phase, player: Player) {
        self.clear_shape_at_mouse_pointer();
        let pt = self.mouse_pos_snapping_to_possible_moves();

        self.shape_at_mouse_pointer = match phase {
            Phase::PlaceRing | Phase::MoveRing(_) => {
                Some(Box::new(PieceShape::new_ring_at_point(pt, player)))
            }
            Phase::PlaceMarker => Some(Box::new(PieceShape::new_marker_at_point(pt, player))),
            Phase::RemoveRun => None,
            Phase::RemoveRing => None,
            Phase::PlayerWon(_) => None,
        };

        if self.shape_at_mouse_pointer.is_none() {
            return;
        }

        self.shape_at_mouse_pointer
            .as_mut()
            .map(|s| s.set_state(ShapeState::AtMousePointer));
    }

    fn interactive_highlight(&mut self, phase: Phase) {
        match phase {
            Phase::MoveRing(from) => {
                if let Some(to) = self.mouse_position_to_legal_field(Some(0.09)) {
                    let pt0: Point = from.into();
                    let pt1: Point = to.into();
                    draw_line(pt0.0, pt0.1, pt1.0, pt1.1, 0.2, BLUE);
                }
            }
            _ => (),
        }
    }
}
