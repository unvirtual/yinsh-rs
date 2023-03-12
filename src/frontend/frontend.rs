use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use crate::common::coord::*;
use crate::core::actions::*;
use crate::core::board::*;
use crate::core::board::*;
use crate::core::entities::*;
use crate::core::game::*;
use crate::core::state::*;
use crate::frontend::animation::FlipAnimation;
use crate::frontend::animation::MoveAnimation;
use crate::frontend::animation::RemoveAnimation;
use crate::frontend::mouse;
use macroquad::hash;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use macroquad::ui::widgets;
use macroquad::ui::Ui;
use macroquad::ui::widgets::Group;

use super::controller::Controller;
use super::controller::ElementId;
use super::element::*;
use super::mouse::MouseHandler;
use super::primitives::build_grid_lines;
use super::primitives::Event;
use super::primitives::Message;

pub type ShapeId = usize;

mod consts {
    use macroquad::prelude::*;

    const BLACK_PLAYER_COLOR: Color = BLACK;
    const WHITE_PLAYER_COLOR: Color = WHITE;
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum UiStatus {
    Idle,
    Busy,
    UpdateRequest,
}

pub struct Frontend {
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
    ui_actions: Vec<UiAction>,
    run_bboxes: Vec<ElementId>,
    pub ui_status: UiStatus,
    update_request: bool,
    white_ring_slots: [Point; 3],
    black_ring_slots: [Point; 3],
}

impl Frontend {
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

        Frontend {
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
            ui_actions: vec![],
            run_bboxes: vec![],
            ui_status: UiStatus::Idle,
            update_request: true,
            white_ring_slots: [
                Point(-radius, -radius),
                Point(-radius + 1., -radius),
                Point(-radius + 2., -radius),
            ],
            black_ring_slots: [
                Point(radius, radius),
                Point(radius - 1., radius),
                Point(radius - 2., radius),
            ],
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

        if mouse_event.right_clicked {
            println!("Right mouse clicked");
            self.ui_actions.push(UiAction::Undo);
        }
    }

    fn add_legal_move_highlights(&mut self, state: &State) {
        self.legal_moves.iter().for_each(|action| {
            let coord = action.coord();
            self.controller
                .add_element(Box::new(FieldMarker::new(coord, 0.1, 0.3, 1)));
        });
    }

    fn add_ring_element(&mut self, c: HexCoord, player: Player) {
        let mut element = Box::new(PieceElement::new_ring_at_coord(c, player, 1));
        match self.phase {
            Phase::RemoveRing => element.set_state(ShapeState::Hoverable),
            _ => (),
        }
        self.controller.add_element(element);
    }

    fn add_marker_element(&mut self, c: HexCoord, player: Player, state: &State) {
        let runs = state.current_player_runs();
        if self.phase == Phase::RemoveRun && runs.iter().flatten().find(|&x| *x == c).is_some() {
            return;
        }
        let element = Box::new(PieceElement::new_marker_at_coord(c, player, 1));
        self.controller.add_element(element);
    }

    fn add_run_elements(&mut self, r: &Vec<HexCoord>, z_value: i32) {
        let mut box_element = Box::new(RunBBoxElement::from_segment_coords(
            r[0],
            *r.last().unwrap(),
            0.5,
            z_value,
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

    fn add_mouse_element(&mut self, mouse_pos: Point) {
        match self.phase {
            Phase::MoveRing(from) => {
                println!("Adding mouse element");
                let mut element = Box::new(PieceElement::new_ring_at_point(
                    mouse_pos,
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

    fn add_won_rings(&mut self, state: &State) {
        for i in 0..state.points_black {
            let pt = self.black_ring_slots[i];
            let element = Box::new(PieceElement::new_ring_at_point(pt, Player::Black, 1));
            self.controller.add_element(element);
        }
        for i in 0..state.points_white {
            let pt = self.white_ring_slots[i];
            let element = Box::new(PieceElement::new_ring_at_point(pt, Player::White, 1));
            self.controller.add_element(element);
        }
    }

    fn create_animations(&mut self, state: &State) -> HashSet<HexCoord> {
        let mut skip_coords = HashSet::new();
        for sc in &state.last_state_change() {
            match sc {
                StateChange::RingPlaced(player, c) => {
                    skip_coords.insert(*c);
                    let mut element = Box::new(PieceElement::new_ring_at_coord(*c, *player, 1));
                    self.controller.add_element(element);
                }
                StateChange::RingMoved(player, from, to) => {
                    if player == &Player::Black {
                        skip_coords.insert(*to);
                        let animation =
                            MoveAnimation::new_box(Point::from(*from), Point::from(*to));
                        let element = AnimatedPieceElement::ring(*player, *from, 1, animation);
                        self.controller.add_element(Box::new(element));
                    }
                }
                StateChange::MarkerFlipped(c) => {
                    skip_coords.insert(*c);
                    let player = state.board.belongs_to(c).unwrap();
                    let start_color = if player == Player::White {
                        BLACK
                    } else {
                        WHITE
                    };
                    let end_color = if player == Player::White {
                        WHITE
                    } else {
                        BLACK
                    };
                    let mut element = Box::new(AnimatedPieceElement::new(
                        PieceElement::new_marker_at_coord(*c, player, 1),
                        FlipAnimation::new_box(start_color, end_color),
                    ));
                    self.controller.add_element(element);
                }
                StateChange::MarkerPlaced(player, c) => {
                    skip_coords.insert(*c);
                    let mut element = Box::new(PieceElement::new_marker_at_coord(*c, *player, 1));
                    self.controller.add_element(element);
                }
                StateChange::MarkerRemoved(player, c) => {
                    skip_coords.insert(*c);
                    let mut element = Box::new(AnimatedPieceElement::new(
                        PieceElement::new_marker_at_coord(*c, *player, 1),
                        RemoveAnimation::new_box(1.2),
                    ));
                    self.controller.add_element(element);
                }
                StateChange::RingRemoved(player, c) => {
                    if state.current_phase == Phase::PlaceMarker {
                        skip_coords.insert(*c);
                        let to_pt = if *player == Player::White {
                            self.white_ring_slots[state.points_white - 1]
                        } else {
                            self.black_ring_slots[state.points_black - 1]
                        };
                        let mut element = Box::new(AnimatedPieceElement::new(
                            PieceElement::new_ring_at_coord(*c, *player, 1),
                            MoveAnimation::new_box(Point::from(*c), to_pt),
                        ));
                        self.controller.add_element(element);
                    }
                }
            }
        }
        skip_coords
    }

    fn update_from_state(&mut self, state: &State) {
        self.current_player = state.current_player;
        self.phase = state.current_phase;

        self.legal_moves = state.legal_moves();

        self.controller.clear_all();

        self.add_legal_move_highlights(state);

        let runs = state.current_player_runs();

        let skip_coords = self.create_animations(state);

        for player in [Player::White, Player::Black] {
            state.board.player_rings(player).for_each(|c| {
                if !skip_coords.contains(c) {
                    self.add_ring_element(*c, player);
                }
            });

            for c in state.board.player_markers(player) {
                if !skip_coords.contains(c) {
                    self.add_marker_element(*c, player, state);
                }
            }
        }
        for (i, r) in runs.iter().enumerate() {
            self.add_run_elements(r, (i + 3) as i32);
        }

        self.add_won_rings(&state);

        self.mouse_handler.update();
        let mouse_event = self.mouse_handler.has_message(None);
        self.add_mouse_element(mouse_event.pos);
    }
}

impl View for Frontend {
    fn request_update(&mut self) {
        self.update_request = true;
    }

    fn tick(&mut self, state: &State) -> UiAction {
        self.ui_actions.clear();

        if self.ui_status == UiStatus::Idle && self.update_request {
            println!("PDATED STATE");
            self.update_from_state(state);
            self.update_request = false;
        }
        clear_background(LIGHTGRAY);

        self.set_camera();

        self.draw_grid();

        self.mouse_handler.update();
        let mouse_event = self.mouse_handler.has_message(Some(&self.legal_moves));
        self.controller.schedule_event(Event::Mouse(mouse_event));

        self.controller.handle_events();

        self.controller.render();
        self.ui_actions = self.controller.get_actions();
        self.update_user_actions();

        if self
            .ui_actions
            .iter()
            .filter(|&a| a == &UiAction::AnimationInProgress)
            .count()
            != 0
        {
            self.ui_status = UiStatus::Busy;
            return UiAction::Busy;
        } else {
            self.ui_status = UiStatus::Idle;
        }
        //println!("{:?}", self.ui_status);
        self.ui_actions.retain(|a| match a {
            UiAction::ActionAtCoord(_) | UiAction::Undo => true,
            _ => false,
        });

        self.ui_actions.pop().unwrap_or(UiAction::NoAction)
    }

    // Idle -> tick --> None -- no update
    // Idle --> tick --> None --> user action registered --> self.transistion --> tick --> TransistionInProgress --> tick --> WaitingForUpdate --> self.Update --> Idle --> ...

    fn invalid_action(&self) {
        println!("MCFrontend: Invalid action");
    }

    fn set_interactive(&mut self, flag: bool) {}
}
