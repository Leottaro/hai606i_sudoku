use env_logger::Env;
use macroquad::prelude::*;
use simple_sudoku::{Sudoku, SudokuDisplay};
use std::{thread, time};

mod simple_sudoku;
mod tests;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    #[cfg(debug_assertions)]
    debug!("Debug activé");
    let mut sudoku = Sudoku::parse_file("sudoku-3-64-9.txt").unwrap();
    println!("{}", sudoku);
    let mut sudoku_display = SudokuDisplay::new(&mut sudoku);

    let font = load_ttf_font("./res/font/RobotoMono-Thin.ttf")
        .await
        .unwrap();
    let temps = time::Duration::from_millis(100);

    while sudoku_display.is_valid().is_ok() && !sudoku_display.is_solved() {
        match sudoku_display.rule_solve(None) {
            Ok(0) => {
                println!("Sudoku solved!");
            }
            Ok(_) => (),
            Err(((x1, y1), (x2, y2))) => {
                println!("Error: ({}, {}) and ({}, {})", x1, y1, x2, y2);
            }
        }

        sudoku_display.run(font.clone()).await;
        next_frame().await;
        thread::sleep(temps);
    }
}
