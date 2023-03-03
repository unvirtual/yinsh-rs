use crate::common::coord::*;
use crate::core::state::*;
use crate::core::entities::*;
use crate::core::ai::*;

pub enum UserAction {
    ActionAtCoord(HexCoord),
    Undo,
}

pub trait View {
    fn poll_user_action(&self) -> Option<UserAction>;
    fn invalid_action(&self);
    fn update(&mut self, state: &State);
    fn render(&self);
}

pub struct Game {
    state: State,
    view: Box<dyn View>,
    human_player: Player,
    ai: RandomAI,
}

impl Game {
    pub fn new(human_player: Player, view: Box<dyn View>) -> Self {
        Game {
            state: State::new(),
            view,
            human_player,
            ai: RandomAI::new(human_player.other(), 4),
        }
    }

    pub fn tick(&mut self) {
        if self.state.current_player == self.human_player {
            if self.poll_user_action_and_execute() {
                self.view.update(&self.state);
            } else {
                //self.view.invalid_action();
            }
        } else {
            self.ai.turn(&mut self.state);
            self.view.update(&self.state);
        }
        self.view.render();
    }

    fn poll_user_action_and_execute(&mut self) -> bool {
        if let Some(user_action) = self.view.poll_user_action() {
            println!("Received user action");
            match user_action {
                UserAction::ActionAtCoord(coord) => self.state.execute_for_coord(&coord),
                UserAction::Undo => self.state.undo(),
            }
        } else {
            false
        }
    }
}