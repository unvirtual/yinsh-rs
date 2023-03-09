use crate::common::coord::*;
use crate::core::board::*;
use crate::core::entities::*;

use super::actions::*;

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

    pub fn legal_moves(&self) -> Vec<Action> {
        match self.current_phase {
            Phase::PlaceRing => self
                .board
                .board_coords()
                .iter()
                .filter(|x| self.board.occupied(x).is_none())
                .map(|c| Action::from(PlaceRing { pos: *c }))
                .collect(),
            Phase::PlaceMarker => self
                .board
                .player_rings(self.current_player)
                .map(|c| Action::from(PlaceMarker { pos: *c }))
                .collect::<Vec<Action>>(),
            Phase::MoveRing(from) => self
                .board
                .ring_targets(&from)
                .iter()
                .map(|c| {
                    Action::from(MoveRing {
                        player: self.current_player,
                        from: from,
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
                        pos: run[0],
                    })
                })
                .collect(),
            Phase::RemoveRing => self
                .board
                .player_rings(self.current_player)
                .map(|c| {
                    Action::from(RemoveRing {
                        player: self.current_player,
                        pos: *c,
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
            self.last_state_change = m.undo(self);
            return true;
        }
        false
    }

    pub fn execute_for_coord(&mut self, coord: &HexCoord) -> bool {
        // cache??
        println!("Trying action for {:?}", coord);
        if let Some(some_move) = self.legal_moves().into_iter().find(|m| m.coord() == *coord) {
            println!("move found: {:?}", some_move);
            if !some_move.is_legal(self) {
                println!("BUT ILLEGAL");
                return false;
            }
            println!("EXECUTING");
            self.last_state_change = some_move.execute(self);
            self.history.push(some_move);
            return true;
        }
        false
    }

    fn last_state_change(&self) -> Vec<StateChange> {
        self.last_state_change.clone()
    }

    fn current_player_runs(&self) -> &Vec<Vec<HexCoord>> {
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
