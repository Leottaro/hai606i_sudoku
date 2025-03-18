#![allow(dead_code)] // no warning due to unused code

use hai606i_sudoku::{
    database::Database,
    simple_sudoku::{Sudoku, SudokuDisplay},
};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Sudoku".to_owned(),
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let font = load_ttf_font("./res/font/RobotoMono-Thin.ttf")
        .await
        .unwrap();
    let database = Database::connect();
    let mut sudoku_display = SudokuDisplay::new(Sudoku::new(3), font.clone(), database).await;

    loop {
        sudoku_display.run(font.clone()).await;
        next_frame().await;
    }
}
