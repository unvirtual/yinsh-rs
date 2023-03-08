use crate::common::coord::*;
use crate::core::actions::*;
use crate::core::ai::*;
use crate::core::board::*;
use crate::core::entities::*;
use crate::core::state::*;

#[derive(Debug, Clone)]
pub enum UserAction {
    ActionAtCoord(HexCoord),
    Undo,
}

pub trait View {
    fn poll_user_action(&mut self) -> Vec<UserAction>;
    fn invalid_action(&self);
    fn update(&mut self, state: &State);
    fn set_interactive(&mut self, flag: bool);
    fn render(&mut self);
}

pub struct Game {
    state: State,
    view: Box<dyn View>,
    human_player: Player,
    ai: RandomAI,
}

impl Game {
    pub fn new(human_player: Player, view: Box<dyn View>, board: Board) -> Self {
        let mut game = Game {
            state: State::new(board),
            view,
            human_player,
            ai: RandomAI::new(human_player.other(), 1),
        };
        game.view.update(&game.state);
        game
    }

    pub fn tick(&mut self) {
        self.view.render();
        if self.state.current_player == self.human_player {
            self.view.set_interactive(true);
            if self.poll_user_action_and_execute() {
                self.view.update(&self.state);
            } else {
                //self.view.invalid_action();
            }
        } else {
            self.view.set_interactive(false);
            self.ai.turn(&mut self.state);
            self.view.update(&self.state);
        }
    }

    fn poll_user_action_and_execute(&mut self) -> bool {
        let actions = self.view.poll_user_action();
        if actions.len() > 0 {
            println!("Game: got actions: {:?}", actions);
        }
        // any here is potentially important: only the first successful action will be used, which prevents multiple actions to be triggered (which I guess is impossible, but I havent checked)
        actions.iter().any(|user_action| match user_action {
            UserAction::ActionAtCoord(coord) => self.state.execute_for_coord(&coord),
            UserAction::Undo => self.state.undo(),
        })
    }
}
