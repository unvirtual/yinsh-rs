pub mod common;
pub mod core;
pub mod frontend;

use crate::core::entities::Piece;

use crate::core::board::Board;
use crate::core::entities::Player;
use crate::core::game::Game;

use common::coord::HexCoord;
use frontend::mcview::MCFrontend;
use macroquad::prelude::*;
use macroquad::window::Conf;

fn window_conf() -> Conf {
    Conf {
        window_title: "yinsh".to_owned(),
        window_width: 1024,
        window_height: 1024,
        high_dpi: true,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut board = Board::new();

    board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(-2, 0));
    board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(-1, 0));
    board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(0, 0));
    board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(1, 0));
    board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(2, 0));
    let mut frontend = MCFrontend::new(&board, 1024, 1024, 1., 1.);
    let mut game = Game::new(Player::White, Box::new(frontend), board);

    loop {
        game.tick();
        next_frame().await
    }
}
