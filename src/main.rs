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
    debug!("Hello, world!");
    let mut sudoku = Sudoku::parse_file("sudoku-3-facile-1.txt").unwrap();
    println!("{}", sudoku);
    sudoku.display_possibilities();
    let mut sudoku_display = SudokuDisplay::new(&mut sudoku);

    let font = load_ttf_font("./res/font/RobotoMono-Thin.ttf")
        .await
        .unwrap();
    let temps = time::Duration::from_millis(100);

    loop {
        match sudoku_display.rule_solve() {
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
