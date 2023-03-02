use enum_dispatch::enum_dispatch;

use crate::game::board::*;
use crate::game::coord::*;
use crate::game::entities::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Phase {
    PlaceRing,
    PlaceMarker,
    MoveRing(Coord),
    RemoveRun,
    RemoveRing,
    PlayerWon(Player),
}

#[derive(Clone)]
pub struct State {
    pub board: Board,
    pub current_player: Player,
    pub current_phase: Phase,
    pub points_white: usize,
    pub points_black: usize,

    pub runs_white: Vec<Vec<Coord>>,
    pub runs_black: Vec<Vec<Coord>>,
    pub actions: Vec<Action>,
}

impl State {
    pub fn new() -> Self {
        State {
            board: Board::new(),
            current_player: Player::White,
            current_phase: Phase::PlaceRing,
            points_white: 0,
            points_black: 0,
            runs_white: vec![],
            runs_black: vec![],
            actions: vec![],
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

    pub fn next(&self, coord: Coord) {
        todo!();
    }

    fn current_player_runs(&self) -> &Vec<Vec<Coord>> {
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

    pub fn get_run(&self, player: &Player, idx: usize) -> Option<&Vec<Coord>> {
        match player {
            Player::White => self.runs_white.get(idx),
            Player::Black => self.runs_black.get(idx),
        }
    }

    pub fn is_valid_run(&self, player: &Player, run: &Vec<Coord>) -> bool {
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

#[enum_dispatch]
pub trait Command {
    fn is_legal(&self, game: &State) -> bool;
    fn execute(&self, game: &mut State);
    fn undo(&self, game: &mut State);
    fn coord(&self) -> Coord;
}

#[enum_dispatch(Command)]
#[derive(Debug, Clone)]
pub enum Action {
    PlaceRing,
    PlaceMarker,
    MoveRing,
    RemoveRun,
    RemoveRing,
}

#[derive(Debug, Clone)]
pub struct PlaceRing {
    pos: Coord,
}

#[derive(Debug, Clone)]
pub struct PlaceMarker {
    pos: Coord,
}

#[derive(Debug, Clone)]
pub struct MoveRing {
    from: Coord,
    to: Coord,
    player: Player,
}

#[derive(Debug, Clone)]
pub struct RemoveRun {
    run_idx: usize,
    run: Vec<Coord>,
    pos: Coord,
}

#[derive(Debug, Clone)]
pub struct RemoveRing {
    pos: Coord,
    player: Player,
}

impl Command for PlaceRing {
    fn is_legal(&self, game: &State) -> bool {
        game.at_phase(&Phase::PlaceRing) && game.board.free_board_field(&self.pos)
    }

    fn execute(&self, game: &mut State) {
        let piece = Piece::Ring(game.current_player);
        game.board.place_unchecked(&piece, &self.pos);

        if game.board.rings().count() > 9 {
            game.set_phase(Phase::PlaceMarker);
        }

        game.next_player();
        game.actions.push(Action::from(self.clone()));
    }

    fn undo(&self, game: &mut State) {
        game.board.remove(&self.pos);
        game.set_phase(Phase::PlaceRing);
        game.next_player();
        game.actions.pop();
    }

    fn coord(&self) -> Coord {
        self.pos
    }
}

impl Command for PlaceMarker {
    fn is_legal(&self, game: &State) -> bool {
        game.at_phase(&Phase::PlaceMarker)
            && game.board.player_ring_at(&self.pos, &game.current_player)
    }

    fn execute(&self, game: &mut State) {
        let piece = Piece::Marker(game.current_player);
        game.board.place_unchecked(&piece, &self.pos);
        game.set_phase(Phase::MoveRing(self.pos));
        game.actions.push(Action::from(self.clone()));
    }

    fn undo(&self, game: &mut State) {
        let piece = Piece::Ring(game.current_player);
        game.board.place_unchecked(&piece, &self.pos);
        game.set_phase(Phase::PlaceMarker);
        game.actions.pop();
    }

    fn coord(&self) -> Coord {
        self.pos
    }
}

impl Command for MoveRing {
    fn is_legal(&self, game: &State) -> bool {
        if !game.at_phase(&Phase::MoveRing(self.from)) {
            return false;
        }
        return game
            .board
            .ring_targets(&self.from)
            .iter()
            .find(|&c| c == &self.to)
            .is_some();
    }

    fn execute(&self, game: &mut State) {
        let piece = Piece::Ring(game.current_player);
        game.board.place_unchecked(&piece, &self.to);
        game.board.flip_between(&self.from, &self.to);

        game.compute_runs();

        if game.board.runs(&game.current_player).len() > 0 {
            game.set_phase(Phase::RemoveRun);
        } else if game.board.runs(&game.current_player.other()).len() > 0 {
            game.set_phase(Phase::RemoveRun);
            game.next_player();
        } else {
            game.set_phase(Phase::PlaceMarker);
            game.next_player();
        }
        game.actions.push(Action::from(self.clone()));
    }

    fn undo(&self, game: &mut State) {
        game.board.remove(&self.to);
        game.board.flip_between(&self.from, &self.to);
        game.current_player = self.player;
        game.set_phase(Phase::MoveRing(self.from));
        game.compute_runs();
        game.actions.pop();
    }

    fn coord(&self) -> Coord {
        self.to
    }
}

impl Command for RemoveRun {
    fn is_legal(&self, game: &State) -> bool {
        game.at_phase(&Phase::RemoveRun) && game.is_valid_run(&game.current_player, &self.run)
    }

    fn execute(&self, game: &mut State) {
        self.run.iter().for_each(|c| {
            game.board.remove(c);
        });

        game.compute_runs();
        game.set_phase(Phase::RemoveRing);
        game.actions.push(Action::from(self.clone()));
    }

    fn undo(&self, game: &mut State) {
        game.set_phase(Phase::RemoveRun);
        let marker = Piece::Marker(game.current_player);
        self.run.iter().for_each(|c| {
            game.board.place_unchecked(&marker, c);
        });
        game.compute_runs();
        game.actions.pop();
    }

    fn coord(&self) -> Coord {
        self.pos
    }
}

impl Command for RemoveRing {
    fn is_legal(&self, game: &State) -> bool {
        game.at_phase(&Phase::RemoveRing)
            && game.board.player_ring_at(&self.pos, &game.current_player)
            && game.current_player == self.player
    }

    fn execute(&self, game: &mut State) {
        game.board.remove(&self.pos);

        let current_player = game.current_player;
        game.inc_score(&current_player);

        if game.get_score(&current_player) == 3 {
            game.set_phase(Phase::PlayerWon(current_player));
            return;
        }

        if game.has_run(&game.current_player) {
            game.set_phase(Phase::RemoveRun);
            return;
        }

        game.next_player();

        if game.has_run(&game.current_player) {
            game.set_phase(Phase::RemoveRun);
        } else {
            game.set_phase(Phase::PlaceMarker);
        }
        game.actions.push(Action::from(self.clone()));
    }

    fn undo(&self, game: &mut State) {
        game.current_player = self.player;
        game.dec_score(&self.player);
        game.set_phase(Phase::RemoveRing);
        let ring = Piece::Ring(game.current_player);
        game.board.place_unchecked(&ring, &self.pos);
        game.actions.pop();
    }

    fn coord(&self) -> Coord {
        self.pos
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_place_ring() {
        let mut game = State::new();
        game.current_player = Player::White;

        let c = Coord::new(2, 4);
        let action = PlaceRing { pos: c };

        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert_eq!(game.board.rings().count(), 1);
        assert!(game.board.player_ring_at(&c, &Player::White));
        assert_eq!(game.current_player, Player::Black);
        assert_eq!(game.current_phase, Phase::PlaceRing);

        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_place_ring_on_occupied_not_allowed() {
        let mut game = State::new();
        game.current_player = Player::White;

        let c = Coord::new(2, 4);
        game.board
            .place_unchecked(&Piece::Marker(Player::White), &c);

        let action = PlaceRing { pos: c };

        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_place_ring_in_wrong_phase_not_allowed() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let c = Coord::new(2, 4);
        let action = PlaceRing { pos: c };

        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_place_ring_undo() {
        let mut game = State::new();
        game.current_player = Player::White;

        let occupied = Coord::new(-1, 1);
        game.board
            .place_unchecked(&Piece::Marker(Player::White), &occupied);

        let c = Coord::new(2, 4);
        let action = PlaceRing { pos: c };

        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert_eq!(game.board.rings().count(), 1);
        assert_eq!(game.board.markers().count(), 1);

        action.undo(&mut game);

        assert!(game.board.occupied(&c).is_none());
        assert!(game.board.occupied(&occupied).is_some());
        assert_eq!(game.current_phase, Phase::PlaceRing);
        assert_eq!(game.current_player, Player::White);

        assert_eq!(game.board.rings().count(), 0);
        assert_eq!(game.board.markers().count(), 1);
    }

    #[test]
    fn test_place_marker() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let c = Coord::new(2, 4);

        let action = PlaceMarker { pos: c };
        game.board.place_unchecked(&Piece::Ring(Player::White), &c);
        assert!(action.is_legal(&game));

        action.execute(&mut game);

        assert_eq!(game.board.markers().count(), 1);
        assert_eq!(game.board.rings().count(), 0);
        assert!(game.board.player_marker_at(&c, &Player::White));
        assert_eq!(game.current_player, Player::White);
        matches!(game.current_phase, Phase::MoveRing(_));
    }

    #[test]
    fn test_place_marker_without_ring() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let c = Coord::new(2, 4);
        let action = PlaceMarker { pos: c };

        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_place_marker_wrong_player_ring() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let c = Coord::new(2, 4);
        game.board.place_unchecked(&Piece::Ring(Player::Black), &c);
        let action = PlaceMarker { pos: c };
        assert!(!action.is_legal(&game));

        game.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = PlaceMarker { pos: c };
        assert!(action.is_legal(&game));
    }

    #[test]
    fn test_place_marker_in_wrong_phase_not_allowed() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::MoveRing(Coord::new(0, 0)));

        let c = Coord::new(2, 4);
        let action = PlaceMarker { pos: c };

        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_place_marker_undo() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let c = Coord::new(2, 4);
        game.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = PlaceMarker { pos: c };
        assert!(action.is_legal(&game));

        assert!(action.is_legal(&game));
        action.execute(&mut game);
        action.undo(&mut game);

        assert!(game.board.player_ring_at(&c, &Player::White));
        assert_eq!(game.current_phase, Phase::PlaceMarker);
        assert_eq!(game.current_player, Player::White);
    }

    #[test]
    fn test_move_ring_without_run() {
        let mut game = State::new();
        game.current_player = Player::White;
        let from_coord = Coord::new(-1, -2);

        game.set_phase(Phase::MoveRing(from_coord));

        let to_coord = Coord::new(-1, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };

        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert_eq!(game.board.rings().count(), 1);
        assert!(game.board.player_ring_at(&to_coord, &Player::White));
        assert_eq!(game.current_player, Player::Black);
        assert_eq!(game.current_phase, Phase::PlaceMarker);
    }

    #[test]
    fn test_move_ring_in_wrong_phase() {
        let mut game = State::new();
        game.current_player = Player::White;
        let from_coord = Coord::new(-1, -2);

        game.set_phase(Phase::PlaceMarker);

        // not connected
        let to_coord = Coord::new(0, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_move_ring_to_illegal_field_not_allowed() {
        let mut game = State::new();
        game.current_player = Player::White;
        let from_coord = Coord::new(-1, -2);

        game.set_phase(Phase::MoveRing(from_coord));

        // not connected
        let to_coord = Coord::new(0, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(!action.is_legal(&game));

        // marker occupied
        let to_coord = Coord::new(-1, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&game));
        game.board
            .place_unchecked(&Piece::Marker(Player::Black), &to_coord);
        assert!(!action.is_legal(&game));

        // ring occupied
        let to_coord = Coord::new(-1, -4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&game));
        game.board
            .place_unchecked(&Piece::Ring(Player::Black), &to_coord);
        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_move_ring_flips_markers() {
        let mut game = State::new();
        game.current_player = Player::White;
        let from_coord = Coord::new(-2, 0);

        game.set_phase(Phase::MoveRing(from_coord));
        game.board
            .place_unchecked(&Piece::Marker(Player::Black), &Coord::new(0, 0));
        game.board
            .place_unchecked(&Piece::Marker(Player::White), &Coord::new(1, 0));

        // not connected
        let to_coord = Coord::new(2, 0);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert!(game
            .board
            .player_marker_at(&Coord::new(0, 0), &Player::White));
        assert!(game
            .board
            .player_marker_at(&Coord::new(1, 0), &Player::Black));
        assert!(game.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_move_ring_creates_run_from_placement() {
        let mut game = State::new();
        game.current_player = Player::White;
        let from_coord = Coord::new(-2, 0);

        game.set_phase(Phase::MoveRing(from_coord));
        for i in -2..=2 {
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &Coord::new(i, 0));
        }
        assert!(!game.has_run(&Player::White));

        // not connected
        let to_coord = Coord::new(-2, 1);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert!(game.has_run(&Player::White));
        assert_eq!(game.current_player, Player::White);
        assert_eq!(game.current_phase, Phase::RemoveRun);
        assert!(game.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_move_ring_creates_run_from_flip() {
        let mut game = State::new();
        game.current_player = Player::White;
        let from_coord = Coord::new(-2, -1);

        game.set_phase(Phase::MoveRing(from_coord));
        game.board
            .place_unchecked(&Piece::Marker(Player::Black), &Coord::new(-2, 0));
        for i in -1..=2 {
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &Coord::new(i, 0));
        }
        assert!(!game.has_run(&Player::White));

        // not connected
        let to_coord = Coord::new(-2, 1);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert!(game.has_run(&Player::White));
        assert_eq!(game.current_player, Player::White);
        assert_eq!(game.current_phase, Phase::RemoveRun);
        assert!(game.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_move_ring_undo() {
        let mut game = State::new();
        game.current_player = Player::White;
        let from_coord = Coord::new(-2, -1);

        game.set_phase(Phase::MoveRing(from_coord));
        game.board
            .place_unchecked(&Piece::Marker(Player::Black), &Coord::new(-2, 0));
        for i in -1..=2 {
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &Coord::new(i, 0));
        }
        assert!(!game.has_run(&Player::White));

        // not connected
        let to_coord = Coord::new(-2, 1);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert!(game.has_run(&Player::White));
        assert_eq!(game.current_player, Player::White);
        assert_eq!(game.current_phase, Phase::RemoveRun);

        action.undo(&mut game);
        assert!(!game.has_run(&Player::White));
        assert_eq!(game.current_player, Player::White);
        assert_eq!(game.current_phase, Phase::MoveRing(from_coord));
        assert!(!game.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_remove_run() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::RemoveRun);

        let mut run = vec![];

        for i in -2..=2 {
            let c = Coord::new(i, 0);
            run.push(c);
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        game.compute_runs();
        for c in run.iter() {
            assert!(game.board.player_marker_at(c, &Player::White));
        }
        assert!(game.has_run(&Player::White));
        assert_eq!(game.get_run(&Player::White, 0), Some(&run));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 0,
            pos: run[0],
        };
        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert!(!game.has_run(&Player::White));
        assert_eq!(game.current_player, Player::White);
        assert_eq!(game.current_phase, Phase::RemoveRing);
        for c in run.iter() {
            assert!(!game.board.player_marker_at(c, &Player::White));
        }
    }

    #[test]
    fn test_remove_run_wrong_phase() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let mut run = vec![];

        for i in -2..=2 {
            let c = Coord::new(i, 0);
            run.push(c);
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        game.compute_runs();
        assert!(game.has_run(&Player::White));
        assert_eq!(game.get_run(&Player::White, 0), Some(&run));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 0,
            pos: run[0],
        };
        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_remove_run_illegal_index() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let mut run = vec![];

        for i in -2..=2 {
            let c = Coord::new(i, 0);
            run.push(c);
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        game.compute_runs();
        assert!(game.has_run(&Player::White));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 2,
            pos: run[0],
        };
        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_remove_run_undo() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::RemoveRun);

        let mut run = vec![];

        for i in -2..=2 {
            let c = Coord::new(i, 0);
            run.push(c);
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        game.compute_runs();
        assert!(game.has_run(&Player::White));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 0,
            pos: run[0],
        };
        assert!(action.is_legal(&game));
        action.execute(&mut game);
        assert!(!game.has_run(&Player::White));

        action.undo(&mut game);
        assert!(game.has_run(&Player::White));
        assert_eq!(game.get_run(&Player::White, 0).unwrap(), &run);
        for c in run.iter() {
            assert!(game.board.player_marker_at(c, &Player::White));
        }
        assert_eq!(game.current_phase, Phase::RemoveRun);
        assert_eq!(game.current_player, Player::White);
    }

    #[test]
    fn test_remove_ring() {
        for player in [Player::White, Player::Black] {
            let mut game = State::new();
            game.current_player = player;
            game.set_phase(Phase::RemoveRing);

            let c = Coord::new(2, 3);
            game.board.place_unchecked(&Piece::Ring(player), &c);

            // not connected
            let action = RemoveRing {
                pos: c.clone(),
                player,
            };
            match player {
                Player::White => assert_eq!(game.points_white, 0),
                Player::Black => assert_eq!(game.points_black, 0),
            }

            assert!(action.is_legal(&game));
            action.execute(&mut game);

            assert_eq!(game.current_player, player.other());
            assert_eq!(game.current_phase, Phase::PlaceMarker);
            assert!(!game.board.player_ring_at(&c, &player));
            match player {
                Player::White => assert_eq!(game.points_white, 1),
                Player::Black => assert_eq!(game.points_black, 1),
            }
        }
    }

    #[test]
    fn test_remove_ring_wrong_phase() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::PlaceMarker);

        let c = Coord::new(2, 3);
        game.board.place_unchecked(&Piece::Ring(Player::White), &c);

        // not connected
        let action = RemoveRing {
            pos: c.clone(),
            player: Player::White,
        };
        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_remove_ring_wrong_pos() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::RemoveRing);

        let c = Coord::new(2, 3);
        game.board.place_unchecked(&Piece::Ring(Player::White), &c);

        // not connected
        let action = RemoveRing {
            pos: Coord::new(0, 0),
            player: Player::White,
        };
        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_remove_ring_wrong_player() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::RemoveRing);

        let c = Coord::new(2, 3);
        game.board.place_unchecked(&Piece::Ring(Player::White), &c);

        // not connected
        let action = RemoveRing {
            pos: c.clone(),
            player: Player::Black,
        };
        assert!(!action.is_legal(&game));
    }

    #[test]
    fn test_remove_ring_player_runs() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::RemoveRing);

        let mut run = vec![];

        for i in -2..=2 {
            let c = Coord::new(i, 0);
            run.push(c);
            game.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }

        let c = Coord::new(2, 3);
        game.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = RemoveRing {
            pos: c.clone(),
            player: Player::White,
        };
        // this is already set by previous step
        game.compute_runs();
        assert!(game.has_run(&Player::White));
        assert_eq!(game.points_white, 0);

        // not connected
        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert_eq!(game.current_player, Player::White);
        assert_eq!(game.current_phase, Phase::RemoveRun);
        assert_eq!(game.points_white, 1);
    }

    #[test]
    fn test_remove_ring_other_player_runs() {
        let mut game = State::new();
        game.current_player = Player::White;
        game.set_phase(Phase::RemoveRing);

        let mut run = vec![];

        for i in -2..=2 {
            let c = Coord::new(i, 0);
            run.push(c);
            game.board
                .place_unchecked(&Piece::Marker(Player::Black), &c);
        }

        let c = Coord::new(2, 3);
        game.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = RemoveRing {
            pos: c.clone(),
            player: Player::White,
        };
        // this is already set by previous step
        game.compute_runs();
        assert!(game.has_run(&Player::Black));
        assert_eq!(game.points_white, 0);

        // not connected
        assert!(action.is_legal(&game));
        action.execute(&mut game);

        assert_eq!(game.current_player, Player::Black);
        assert_eq!(game.current_phase, Phase::RemoveRun);
        assert_eq!(game.points_white, 1);
    }

    #[test]
    fn test_remove_ring_undo() {
        for player in [Player::White, Player::Black] {
            let mut game = State::new();
            game.current_player = player;
            game.set_phase(Phase::RemoveRing);

            let c = Coord::new(2, 3);
            game.board.place_unchecked(&Piece::Ring(player), &c);

            // not connected
            let action = RemoveRing {
                pos: c.clone(),
                player,
            };
            match player {
                Player::White => assert_eq!(game.points_white, 0),
                Player::Black => assert_eq!(game.points_black, 0),
            }

            assert!(action.is_legal(&game));
            action.execute(&mut game);

            assert_eq!(game.current_player, player.other());
            assert_eq!(game.current_phase, Phase::PlaceMarker);
            assert!(!game.board.player_ring_at(&c, &player));
            match player {
                Player::White => assert_eq!(game.points_white, 1),
                Player::Black => assert_eq!(game.points_black, 1),
            }

            action.undo(&mut game);
            assert_eq!(game.current_player, player);
            assert_eq!(game.current_phase, Phase::RemoveRing);
            assert!(game.board.player_ring_at(&c, &player));
        }
    }
}
