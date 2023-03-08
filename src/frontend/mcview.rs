use std::collections::HashMap;

use crate::common::coord::*;
use crate::core::actions::*;
use crate::core::board::*;
use crate::core::board::*;
use crate::core::entities::*;
use crate::core::game::*;
use crate::core::state::*;
use crate::frontend::mouse;
use macroquad::prelude::*;

use super::controller::Controller;
use super::controller::ElementId;
use super::element::*;
use super::mouse::MouseHandler;
use super::primitives::build_grid_lines;
use super::primitives::Message;

pub type ShapeId = usize;

pub struct MCFrontend {
    grid_lines: Vec<[HexCoordF; 2]>,
    width: f32,
    height: f32,
    pixel_width: u32,
    pixel_height: u32,
    legal_moves: Vec<Action>,
    phase: Phase,
    current_player: Player,
    controller: Controller,
    mouse_handler: MouseHandler,
    user_actions: Vec<UserAction>,
    run_bboxes: Vec<ElementId>,
}

impl MCFrontend {
    pub fn new(
        board: &Board,
        pixel_width: u32,
        pixel_height: u32,
        w_margin: f32,
        h_margin: f32,
    ) -> Self {
        let radius = board.get_radius();
        let width = (2. * radius + w_margin);
        let height = (2. * radius + h_margin);

        MCFrontend {
            grid_lines: build_grid_lines(radius),
            width: width,
            height: height,
            pixel_width,
            pixel_height,
            legal_moves: vec![],
            phase: Phase::PlaceRing,
            current_player: Player::White,
            controller: Controller::new(),
            mouse_handler: MouseHandler::new(width, height, pixel_width, pixel_height),
            user_actions: vec![],
            run_bboxes: vec![],
        }
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

    fn update_user_actions(&mut self) {
        let mouse_event = self.mouse_handler.has_message(None);
        if mouse_event.left_clicked {
            if let Some(coord) = mouse_event.coord {
                self.user_actions.push(UserAction::ActionAtCoord(coord));
            }
        }

        if mouse_event.right_clicked {
            self.user_actions.push(UserAction::Undo);
        }
    }
}

impl View for MCFrontend {
    fn update(&mut self, state: &State) {
        self.current_player = state.current_player;
        self.phase = state.current_phase;

        self.legal_moves = state.legal_moves();

        self.controller.clear_all();

        let runs = match self.current_player {
            Player::Black => &state.runs_black,
            Player::White => &state.runs_white,
        };

        for player in [Player::White, Player::Black] {
            state.board.player_rings(player).for_each(|c| {
                let mut element = Box::new(PieceElement::new_ring_at_coord(*c, player, 1));
                match self.phase {
                    Phase::RemoveRing => element.set_state(ShapeState::Hoverable),
                    _ => (),
                }
                self.controller.add_element(element);
            });

            for c in state.board.player_markers(player) {
                if self.phase == Phase::RemoveRun
                    && runs.iter().flatten().find(|&x| x == c).is_some()
                {
                    continue;
                }
                let mut element = Box::new(PieceElement::new_marker_at_coord(*c, player, 1));
                self.controller.add_element(element);
            }
        }
        for r in runs {
            let mut box_element = Box::new(RunBBoxElement::from_segment_coords(
                r[0],
                *r.last().unwrap(),
                0.5,
            ));
            box_element.set_coord(r[0]);
            let box_id = self.controller.add_element(box_element);
            self.run_bboxes.push(box_id);
            for c in r {
                let mut element = Box::new(PieceElement::new_marker_at_coord(
                    *c,
                    self.current_player,
                    1,
                ));
                element.set_state(ShapeState::Hoverable);
                let marker_id = self.controller.add_element_inactive(element);
                self.controller.add_subscriber(box_id, marker_id);
            }
        }

        self.mouse_handler.update();
        let mouse_event = self.mouse_handler.has_message(None);
        match self.phase {
            Phase::MoveRing(from) => {
                let mut element = Box::new(PieceElement::new_ring_at_point(
                    mouse_event.pos,
                    self.current_player,
                    10,
                ));
                element.set_state(ShapeState::AtMousePointer);
                self.controller.add_element(element);

                let mut element = Box::new(RingMoveLineElement::new(from.into(), from.into(), -1));
                self.controller.add_element(element);
            }
            _ => (),
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

        self.mouse_handler.update();
        let mouse_event = self.mouse_handler.has_message(Some(&self.legal_moves));

        self.controller.handle_input(&mouse_event);

        self.run_bboxes.iter().for_each(|id| {
            let msgs_filtered = self
                .controller
                .get_messages(
                    |msg| {
                        if let Message::MouseClicked(_) = msg {
                            true
                        } else {
                            false
                        }
                    },
                    *id,
                );
            let msgs = msgs_filtered.iter()
                .map(|msg| {
                    if let Message::MouseClicked(coord) = msg {
                        Some(UserAction::ActionAtCoord(*coord))
                    } else {
                        None
                    }.unwrap()
                }).collect::<Vec<_>>();

            println!("MSGS: {:?}", msgs);
            if msgs.len() > 0 {
                self.user_actions.extend(msgs);
            }
        });
        self.update_user_actions();
        self.controller.render();
    }

    fn poll_user_action(&mut self) -> Vec<UserAction> {
        self.user_actions.drain(0..).collect()
    }

    fn invalid_action(&self) {
        println!("MCFrontend: Invalid action");
    }

    fn set_interactive(&mut self, flag: bool) {}
}
