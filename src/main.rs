pub mod game;
use crate::game::board::*;
use crate::game::screen::*;
use crate::game::game::*;
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
    const width : f32 = 10.;
    const height : f32 = 10.;

    let mut board = Board::new();
    let mut screen = GameCanvas::new(&board, 1024., 1024., width, height);
    let mut game = Game::new();

    loop {
        clear_background(LIGHTGRAY);
        screen.update(&mut game);
        screen.render(&game);
        next_frame().await
    }
}