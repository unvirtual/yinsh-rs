use crate::common::coord::*;
use crate::core::actions::*;
use crate::core::ai::*;
use crate::core::board::*;
use crate::core::entities::*;
use crate::core::state::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    ActionAtCoord(HexCoord),
    Undo,
    RequestUpdate,
    UiUpdated,
    NoAction,
    AnimationFinished,
    AnimationInProgress,
}

pub trait View {
    fn invalid_action(&self);
    fn request_update(&mut self);
    fn set_interactive(&mut self, flag: bool);
    fn tick(&mut self, state: &State) -> UiAction;
}

pub struct Game {
    state: State,
    view: Box<dyn View>,
    human_player: Player,
    ai: RandomAI,
    view_update_scheduled: bool,
}

impl Game {
    pub fn new(human_player: Player, view: Box<dyn View>, board: Board) -> Self {
        let mut game = Game {
            state: State::new(board),
            view,
            human_player,
            ai: RandomAI::new(human_player.other(), 1),
            view_update_scheduled: false,
        };
        game.view.request_update();
        game
    }

    pub fn tick(&mut self) {
        let ui_action = self.view.tick(&mut self.state);

        if self.state.current_player == self.human_player.other() {
            println!("START AI");
            self.ai.turn(&mut self.state);
            self.view.request_update();
            println!("END AI");
            return;
        }

        let successful_action = match ui_action {
            UiAction::ActionAtCoord(coord) => self.state.execute_for_coord(&coord),
            UiAction::Undo => self.state.undo(),
            _ => false,
        };

        if successful_action {
            self.view.request_update();
        }
    }

}
