use std::collections::HashMap;
use std::ops::Deref;

use crate::common::coord::*;
use crate::core::actions::*;
use crate::core::board::*;
use crate::core::board::*;
use crate::core::entities::*;
use crate::core::game::*;
use crate::core::state::*;
use crate::frontend::mouse;
use macroquad::prelude::*;
use macroquad::ui::Ui;

use super::controller::Controller;
use super::controller::ElementId;
use super::element::*;
use super::mouse::MouseHandler;
use super::primitives::build_grid_lines;
use super::primitives::Event;
use super::primitives::Message;

pub type ShapeId = usize;

pub enum UiStatus {
    Idle,
    Busy,
    UpdateRequest,
}

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
    ui_actions: Vec<UiAction>,
    run_bboxes: Vec<ElementId>,
    outstanding_animations: u32,
    update_scheduled: bool,
    start_transition: bool,
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
            ui_actions: vec![],
            run_bboxes: vec![],
            outstanding_animations: 0,
            update_scheduled: false,
            start_transition: false,
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
            self.ui_actions.push(UiAction::Undo);
        }
    }

    fn add_field_markers(&mut self, state: &State) {
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
        let runs = match self.current_player {
            Player::Black => &state.runs_black,
            Player::White => &state.runs_white,
        };
        if self.phase == Phase::RemoveRun && runs.iter().flatten().find(|&x| *x == c).is_some() {
            return;
        }
        let mut element = Box::new(PieceElement::new_marker_at_coord(c, player, 1));
        self.controller.add_element(element);
    }

    fn add_run_elements(&mut self, r: &Vec<HexCoord>) {
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

    fn add_mouse_element(&mut self, mouse_pos: Point) {
        match self.phase {
            Phase::MoveRing(from) => {
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

    fn update_from_state(&mut self, state: &State) {
        self.current_player = state.current_player;
        self.phase = state.current_phase;

        self.legal_moves = state.legal_moves();

        self.controller.clear_all();

        self.add_field_markers(state);

        let runs = match self.current_player {
            Player::Black => &state.runs_black,
            Player::White => &state.runs_white,
        };

        for player in [Player::White, Player::Black] {
            state.board.player_rings(player).for_each(|c| {
                self.add_ring_element(*c, player);
            });

            for c in state.board.player_markers(player) {
                self.add_marker_element(*c, player, state);
            }
        }
        for r in runs {
            self.add_run_elements(r);
        }

        self.mouse_handler.update();
        let mouse_event = self.mouse_handler.has_message(None);
        self.add_mouse_element(mouse_event.pos);
    }
}

impl View for MCFrontend {
    fn request_update(&mut self) {
        self.update_scheduled = true;
        self.start_transition = true;
    }

    fn tick(&mut self, state: &State) -> UiAction {

        // TODO
        // update state, don't draw state_change elements during normal pass and trigger animations form stateChanges.
        // when finished, all shapes  should be at the right position
        // differ between human player and UI --> e.g. MoveRing needs to be animated or not

        if self.start_transition && state.last_state_change.len() > 0 {
            for c in &state.last_state_change {
                let event = match c {
                    //StateChange::RingPlaced(_, coord) => None,
                    //StateChange::RingMoved(_, from, to) => Some(Event::MoveRing(*from, *to)),
                    StateChange::MarkerFlipped(coord) => Some(Event::FlipMarker(*coord)),
                    //StateChange::MarkerPlaced(_, _) => None,
                    //StateChange::MarkerRemoved(_, coord) => Some(Event::RemoveMarker(*coord)),
                    //StateChange::RingRemoved(_, coord) => Some(Event::RemoveRing(*coord)),
                    _ => None,
                };
                if event.is_some() {
                    self.outstanding_animations += 1;
                }
                event.map(|e| self.controller.schedule_event(e));
            }
            self.start_transition = false;
        }

        if self.update_scheduled && self.outstanding_animations == 0 {
            self.update_from_state(state);
            self.update_scheduled = false;
        }
        clear_background(LIGHTGRAY);
        self.set_camera();
        self.draw_grid();

        self.mouse_handler.update();
        let mouse_event = self.mouse_handler.has_message(Some(&self.legal_moves));
        self.controller.schedule_event(Event::Mouse(mouse_event));

        self.controller.handle_events();

        self.update_user_actions();
        self.controller.render();
        self.ui_actions = self.controller.get_actions();
        self.outstanding_animations = self.ui_actions.iter().filter(|&a| a == &UiAction::AnimationInProgress).count() as u32;

        //println!("Outstanding: {}", self.outstanding_animations);
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
