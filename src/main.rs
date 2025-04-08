#![allow(dead_code)] // no warning due to unused code

#[cfg(feature = "database")]
use std::{
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

#[cfg(feature = "database")]
use hai606i_sudoku::database::Database;

use hai606i_sudoku::{carpet_sudoku::{CarpetPattern, CarpetSudoku}, display::SudokuDisplay};
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

    let carpet = CarpetSudoku::new(3, CarpetPattern::Diagonal(3));
    let mut sudoku_display = SudokuDisplay::new(carpet, font.clone()).await;

    #[cfg(feature = "database")]
    let (tx, rx) = mpsc::channel::<Option<Database>>();
    #[cfg(feature = "database")]
    thread::spawn(move || loop {
        let _ = tx.send(Database::connect());
        sleep(Duration::from_secs(5));
    });

	for i in 1..=3 {
		CarpetSudoku::new_carpet(i, i);
	}

    loop {
        #[cfg(feature = "database")]
        if let Ok(db) = rx.try_recv() {
            sudoku_display.set_db(db);
        }
        sudoku_display.run(font.clone()).await;
        next_frame().await;
    }
}
