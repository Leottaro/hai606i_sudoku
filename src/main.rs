#![allow(dead_code)] // no warning due to unused code

use std::{fs, io::stdin};
#[cfg(feature = "database")]
use std::{
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

#[cfg(feature = "database")]
use hai606i_sudoku::database::Database;

use hai606i_sudoku::display::SudokuDisplay;
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
    #[cfg(feature = "database")]
    {
        dotenv::dotenv().ok();
        if std::env::var("DATABASE_URL").is_err() {
            println!("Please enter the mandatory DATABASE_URL (postgresql://<USER>:<USER_PASSWORD>@<DB_IP>/database) :");
            let mut database_url = String::new();
            stdin().read_line(&mut database_url).unwrap();
            database_url = database_url.trim().to_string();

            fs::write(".env", format!("DATABASE_URL={database_url}")).unwrap();
        }
    }

    // env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let font = load_ttf_font("./res/font/RobotoMono-Thin.ttf")
        .await
        .unwrap();

    let mut sudoku_display = SudokuDisplay::new(3, font.clone()).await;

    #[cfg(feature = "database")]
    let (tx, rx) = mpsc::channel::<Option<Database>>();
    #[cfg(feature = "database")]
    thread::spawn(move || loop {
        let _ = tx.send(Database::connect());
        sleep(Duration::from_secs(5));
    });

    loop {
        #[cfg(feature = "database")]
        if let Ok(db) = rx.try_recv() {
            sudoku_display.set_db(db);
        }
        sudoku_display.run(font.clone()).await;
        next_frame().await;
    }
}
