use crate::{common::coord::{Point, HexCoord}, core::entities::Player};

use super::mouse::MouseEvent;

#[derive(PartialEq, Clone, Debug)]
pub enum Message {
    MouseEntered,
    MouseLeft,
    MouseInside,
    ElementMoved(Point),
    ElementShow,
    ElementHide,
    MouseClicked(HexCoord),
    Tick,
    FlipMarker(HexCoord),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    FlipMarker(HexCoord),
    RemoveMarker(HexCoord),
    RemoveRing(HexCoord),
    MoveRing(HexCoord, HexCoord),
    PlaceRing(Player, HexCoord),
}
