use super::{game::*, entities::*};
use rand::thread_rng;
use rand::seq::SliceRandom;

pub struct RandomAI {
    player: Player,
    max_depth: u32,
    pub evaluated_moves: u32,
}

impl RandomAI {
    pub fn new(player: Player, max_depth: u32) -> Self {
        RandomAI {
            max_depth,
            player,
            evaluated_moves: 0,
        }
    }

    pub fn turn(&mut self, game: &mut State) {
        println!("AI turn");
        self.evaluated_moves = 0;
        let mut game_clone = game.clone();
        let moves = game.legal_moves();
        println!("Found legal moves: {:?}", moves.len());
        if game.at_phase(&Phase::PlaceRing) {
            let mut rng = thread_rng();
            let action = moves.choose(&mut rng).unwrap();
            action.execute(game);
            return;
        }

        let mut best_action = None;
        let mut best_score : f32 = f32::NEG_INFINITY;
        self.evaluated_moves += moves.len() as u32;
        for action in moves {
            action.execute(&mut game_clone);
            let score = self.alpha_beta(&mut game_clone, self.max_depth, self.player, None, None);
            if score > best_score {
                best_score = score;
                best_action = Some(action.clone());
            }
            println!("  >> Eval: {:?}, {}", action, score);
            action.undo(&mut game_clone);
        }
        if let Some(action) = best_action {
            action.execute(game);
        }
    }


    fn player_heuristic(&self, game: &State, player: Player) -> f32 {
        let n_markers = game.board.player_markers(player).count();
        let ring_moves: usize = game.board.player_rings(player)
            .map(|c| game.board.ring_targets(c).len())
            .sum::<usize>();
        let points = 100000*game.get_score(&player);

        let connected_3 = 10*game.board.n_connected_markers(&player, 3);
        let connected_4 = 30*game.board.n_connected_markers(&player, 4);
        let run = 1000*game.board.runs(&player).len();

        (n_markers + ring_moves + points + connected_3 + connected_4 + run) as f32
    }

    fn heuristic(&self, game: &State, player: Player) -> f32 {
        self.player_heuristic(game, player) - self.player_heuristic(game, player.other())
    }

    fn alpha_beta(&mut self, game: &mut State, depth: u32, ai_player: Player, alpha: Option<f32>, beta: Option<f32>) -> f32 {
        let mut alpha = alpha.unwrap_or(f32::NEG_INFINITY);
        let mut beta = beta.unwrap_or(f32::INFINITY);

        if game.won_by().is_some() || depth == 0 {
            return self.heuristic(&game, ai_player);
        }

        if game.current_player == ai_player {
    //        println!("     >> AI player");
            let mut best_val = f32::NEG_INFINITY;
            for m in game.legal_moves() {
    //            println!("     >> {:?}", m);
                self.evaluated_moves += 1;

                if !m.is_legal(game) {
                    panic!("ILLEGAL ACTION {:?}", m);
                }
                m.execute(game);
                let value = self.alpha_beta(game, depth - 1, ai_player, Some(alpha), Some(beta));
                m.undo(game);

                best_val = best_val.max(value);
                alpha = alpha.max(best_val);
                if beta <= alpha {
                    break;
                }
            }
            return best_val
        }

        let mut best_val = f32::INFINITY;
        for m in game.legal_moves() {
            self.evaluated_moves += 1;
            //println!("     >> Human player");
            //println!("     >> {:?}", m);
                if !m.is_legal(game) {
                    panic!("ILLEGAL ACTION {:?}", m);
                }
            m.execute(game);
            let value = self.alpha_beta(game, depth - 1, ai_player, Some(alpha), Some(beta));
            m.undo(game);
            best_val = best_val.min(value);
            beta = beta.min(best_val);
            if beta <= alpha {
                break;
            }
        }
        return best_val

    }
}
