pub mod core;
pub mod frontend;
pub mod common;

use crate::core::board::Board;
use crate::core::entities::Player;
use crate::core::game::Game;

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
    let mut frontend = MCFrontend::new(&board, 1024, 1024, 1., 1.);
    let mut game = Game::new(Player::White, Box::new(frontend));

    loop {
        game.tick();
        next_frame().await
    }
}