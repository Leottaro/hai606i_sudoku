use std::{
    io::{stdout, Write},
    ops::AddAssign,
    process::exit,
    sync::{Arc, Mutex},
    thread,
};

use hai606i_sudoku::{
    database::Database,
    simple_sudoku::{Sudoku, SudokuDifficulty},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filled|games>", args[0]);
        std::process::exit(1);
    }

    if args[1].eq("filled") {
        filled();
        exit(0);
    } else if args[1].ne("games") {
        games();
        exit(0);
    }
    eprintln!("Usage: {} <filled|games>", args[0]);
    exit(1);
}

fn filled() {
    let database = Arc::new(Mutex::new(Database::connect().unwrap()));
    let mut total_count: u128 = 0;
    let overlap_count: Arc<Mutex<u128>> = Arc::new(Default::default());
    loop {
        let mut sudokus = Vec::new();
        for _ in 0..100 {
            total_count += 1;
            print!(
                "\r{}/{total_count} overlapped",
                overlap_count.lock().unwrap()
            );
            stdout().flush().unwrap();
            let sudoku_base = Sudoku::generate_full(3);
            let (inserted_sudoku_filled, _sudoku_game) = sudoku_base.to_db();
            sudokus.push(inserted_sudoku_filled);
        }

        let thread_database = Arc::clone(&database);
        let thread_overlap_count = Arc::clone(&overlap_count);
        let thread_sudokus = sudokus.clone();
        thread::spawn(move || {
            let row_inserted = thread_database
                .lock()
                .unwrap()
                .insert_multiple_simple_sudoku_filled(thread_sudokus)
                .unwrap_or_else(|err| {
                    panic!("ERROR COULDN'T INSERT SUDOKU FILLED IN DATABSE: {err}")
                });
            thread_overlap_count
                .lock()
                .unwrap()
                .add_assign(100 - row_inserted as u128);
        });
    }
}

fn games() {
    let database = Arc::new(Mutex::new(Database::connect().unwrap()));
    let difficulties = SudokuDifficulty::iter().collect::<Vec<_>>();
    for (count, &difficulty) in (0_u128..).zip(difficulties.iter().cycle()) {
        println!("\n{count}: difficulty {difficulty}: ");

        let sudoku_base = Sudoku::generate_full(3);
        let sudoku = Sudoku::generate_new(3, difficulty, Some(sudoku_base));

        let thread_database = Arc::clone(&database);
        let (inserted_sudoku_filled, mut inserted_sudoku_game) = sudoku.to_db();
        inserted_sudoku_game.game_difficulty = difficulty as u8;
        thread::spawn(move || {
            thread_database
                .lock()
                .unwrap()
                .insert_simple_sudoku_filled(inserted_sudoku_filled)
                .unwrap_or_else(|err| {
                    if err != diesel::result::Error::AlreadyInTransaction {
                        eprintln!("Error in insert_simple_sudoku_filled: {err}")
                    }
                    panic!("ERROR COULDN'T INSERT SUDOKU FILLED IN DATABSE: {err}")
                });

            thread_database
                .lock()
                .unwrap()
                .insert_simple_sudoku_game(inserted_sudoku_game)
                .unwrap_or_else(|err| {
                    panic!("ERROR COULDN'T INSERT SUDOKU GAME IN DATABSE: {err}")
                });
        });
    }
}
