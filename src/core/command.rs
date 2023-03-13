use enum_dispatch::enum_dispatch;

use crate::common::coord::HexCoord;
use super::actions::*;
use super::state::State;

#[enum_dispatch]
pub trait Command {
    fn is_legal(&self, state: &State) -> bool;
    fn execute(&self, state: &mut State);
    fn undo(&self, state: &mut State);
    fn coord(&self) -> HexCoord;
}
