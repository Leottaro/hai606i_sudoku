use env_logger::Env;
use macroquad::prelude::*;
use simple_sudoku::{Sudoku, SudokuDisplay};

mod simple_sudoku;
mod tests;

#[macro_export]
macro_rules! debug_only {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

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
    //#[cfg(debug_assertions)]
    //debug!("Debug activé");
    let mut sudoku = Sudoku::new(3);
    println!("{}", sudoku);
    let font = load_ttf_font("./res/font/RobotoMono-Thin.ttf")
        .await
        .unwrap();
    let mut sudoku_display = SudokuDisplay::new(&mut sudoku, font.clone()).await;

    loop {
        sudoku_display.run(font.clone()).await;
        next_frame().await;
    }
}
