mod sudoku;
use macroquad::prelude::*;
use sudoku::{simple_sudoku::Sudoku, simple_sudoku_display::SudokuDisplay};

mod tests;

fn window_conf() -> Conf {
    Conf {
        window_title: "Sudoku".to_owned(),
        window_width: 900,
        window_height: 900,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut sudoku = Sudoku::parse_file("test.txt").unwrap();
    println!("{}", sudoku);
    let mut sudoku_display = SudokuDisplay::new(&mut sudoku);

    let font = load_ttf_font("./res/font/thin.ttf")
                .await
                .unwrap();
    loop {
        sudoku_display.run(font.clone()).await;
        next_frame().await
    }
}
