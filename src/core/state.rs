use crate::common::coord::*;
use crate::core::board::*;
use crate::core::entities::*;

use super::actions::*;
use super::command::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Phase {
    PlaceRing,
    PlaceMarker,
    MoveRing(HexCoord),
    RemoveRun,
    RemoveRing,
    PlayerWon(Player),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateChange {
    RingPlaced(Player, HexCoord),
    RingMoved(Player, HexCoord, HexCoord),
    MarkerFlipped(HexCoord),
    MarkerPlaced(Player, HexCoord),
    MarkerRemoved(Player, HexCoord),
    RingRemoved(Player, HexCoord),
}

#[derive(Clone)]
pub struct State {
    pub board: Board,
    pub current_player: Player,
    pub current_phase: Phase,
    pub points_white: usize,
    pub points_black: usize,

    pub runs_white: Vec<Vec<HexCoord>>,
    pub runs_black: Vec<Vec<HexCoord>>,
    pub history: Vec<Action>,
    pub last_state_change: Vec<StateChange>,
}

impl State {
    pub fn new(board: Board) -> Self {
        State {
            board,
            current_player: Player::White,
            current_phase: Phase::PlaceRing,
            points_white: 0,
            points_black: 0,
            runs_white: vec![],
            runs_black: vec![],
            history: vec![],
            last_state_change: vec![],
        }
    }

    pub fn new_action(&mut self) {
        self.last_state_change.clear();
    }

    fn push_state_change(&mut self, state_change: StateChange) {
        self.last_state_change.push(state_change);
    }

    pub fn place_ring(&mut self, player: &Player, coord: &HexCoord) {
        let piece = Piece::Ring(*player);
        let removed = self.board.place_unchecked(&piece, coord);
        self.push_state_change(StateChange::RingPlaced(*player, *coord));

        if let Some(piece) = removed {
            if piece.is_marker() {
                self.push_state_change(StateChange::MarkerRemoved(piece.owner(), *coord));
            }
            if piece.is_ring() {
                self.push_state_change(StateChange::RingRemoved(piece.owner(), *coord));
            }
        }
    }

    pub fn move_ring(
        &mut self,
        player: &Player,
        from: &HexCoord,
        to: &HexCoord,
        remove_from: bool,
        place_to: bool,
    ) {
        if place_to {
            let piece = Piece::Ring(*player);
            self.board.place_unchecked(&piece, to);
        }
        if remove_from {
            self.board.remove(from);
        }
        self.push_state_change(StateChange::RingMoved(*player, *from, *to));
    }

    pub fn remove_ring(&mut self, player: &Player, coord: &HexCoord) {
        self.board.remove(coord);
        self.push_state_change(StateChange::RingRemoved(*player, *coord));
    }

    pub fn place_marker(&mut self, player: &Player, coord: &HexCoord) {
        let piece = Piece::Marker(*player);
        let removed = self.board.place_unchecked(&piece, coord);
        self.push_state_change(StateChange::RingPlaced(*player, *coord));
        if let Some(piece) = removed {
            if piece.is_marker() {
                self.push_state_change(StateChange::MarkerRemoved(piece.owner(), *coord));
            }
            if piece.is_ring() {
                self.push_state_change(StateChange::RingRemoved(piece.owner(), *coord));
            }
        }
    }

    pub fn remove_marker(&mut self, player: &Player, coord: &HexCoord) {
        self.board.remove(coord);
        self.push_state_change(StateChange::MarkerRemoved(*player, *coord));
    }

    pub fn flip_markers(&mut self, from: &HexCoord, to: &HexCoord) {
        let flipped = self.board.flip_between(from, to);
        self.last_state_change
            .extend(flipped.into_iter().map(StateChange::MarkerFlipped));
    }

    pub fn legal_moves(&self) -> Vec<Action> {
        match self.current_phase {
            Phase::PlaceRing => self
                .board
                .board_coords()
                .iter()
                .filter(|x| self.board.occupied(x).is_none())
                .map(|c| Action::from(PlaceRing { coord: *c }))
                .collect(),
            Phase::PlaceMarker => self
                .board
                .player_rings(self.current_player)
                .map(|c| Action::from(PlaceMarker { coord: *c }))
                .collect::<Vec<Action>>(),
            Phase::MoveRing(from) => self
                .board
                .ring_targets(&from)
                .iter()
                .map(|c| {
                    Action::from(MoveRing {
                        player: self.current_player,
                        from,
                        to: *c,
                    })
                })
                .collect(),
            // TODO: this does not always work for multiple simultaneous runs!!
            Phase::RemoveRun => self
                .current_player_runs()
                .iter()
                .enumerate()
                .map(|(idx, run)| {
                    Action::from(RemoveRun {
                        run_idx: idx,
                        run: run.clone(),
                        coord: run[0],
                    })
                })
                .collect(),
            Phase::RemoveRing => self
                .board
                .player_rings(self.current_player)
                .map(|c| {
                    Action::from(RemoveRing {
                        player: self.current_player,
                        coord: *c,
                    })
                })
                .collect::<Vec<Action>>(),
            Phase::PlayerWon(_) => Vec::new(),
        }
    }

    pub fn next(&self, coord: HexCoord) {
        todo!();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(m) = self.history.pop() {
            m.undo(self);
            return true;
        }
        false
    }

    pub fn last_state_change(&self) -> Vec<StateChange> {
        self.last_state_change.clone()
    }

    pub fn current_player_runs(&self) -> &Vec<Vec<HexCoord>> {
        match self.current_player {
            Player::Black => &self.runs_black,
            Player::White => &self.runs_white,
        }
    }

    pub fn next_player(&mut self) {
        self.current_player = self.current_player.other();
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.current_phase = phase;
    }

    pub fn at_phase(&self, phase: &Phase) -> bool {
        self.current_phase == *phase
    }

    pub fn compute_runs(&mut self) {
        self.runs_white = self.board.runs(&Player::White);
        self.runs_black = self.board.runs(&Player::Black);
    }

    pub fn has_run(&self, player: &Player) -> bool {
        match player {
            Player::White => self.runs_white.len() > 0,
            Player::Black => self.runs_black.len() > 0,
        }
    }

    pub fn get_run(&self, player: &Player, idx: usize) -> Option<&Vec<HexCoord>> {
        match player {
            Player::White => self.runs_white.get(idx),
            Player::Black => self.runs_black.get(idx),
        }
    }

    pub fn is_valid_run(&self, player: &Player, run: &Vec<HexCoord>) -> bool {
        match player {
            Player::White => self.runs_white.iter().find(|&r| r == run).is_some(),
            Player::Black => self.runs_black.iter().find(|&r| r == run).is_some(),
        }
    }

    pub fn inc_score(&mut self, player: &Player) {
        match player {
            Player::White => self.points_white += 1,
            Player::Black => self.points_black += 1,
        }
    }

    pub fn dec_score(&mut self, player: &Player) {
        match player {
            Player::White => self.points_white -= 1,
            Player::Black => self.points_black -= 1,
        }
    }

    pub fn get_score(&self, player: &Player) -> usize {
        match player {
            Player::White => self.points_white,
            Player::Black => self.points_black,
        }
    }

    pub fn won_by(&self) -> Option<Player> {
        if let Phase::PlayerWon(player) = self.current_phase {
            return Some(player);
        }
        None
    }
}
