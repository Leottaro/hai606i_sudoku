use std::{
    io::{stdout, Write},
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
        eprintln!("Usage: {} <canonical|games>", args[0]);
        std::process::exit(1);
    }

    if args[1].eq("canonical") {
        canonical();
        exit(0);
    } else if args[1].eq("games") {
        games();
        exit(0);
    }
    eprintln!("Usage: {} <canonical|games>", args[0]);
    exit(1);
}

fn canonical() {
    let mut database = Database::connect().unwrap();
    let mut overlap_count: u128 = 0;
    let mut total_count: u128 = 0;
    let sudokus = Arc::new(Mutex::new(Vec::new()));
    let squares = Arc::new(Mutex::new(Vec::new()));

    let thread_sudokus = Arc::clone(&sudokus);
    let thread_squares = Arc::clone(&squares);
    thread::spawn(move || loop {
        while thread_sudokus.lock().unwrap().len() < 1000 {
            let sudoku_base = Sudoku::generate_full(3);
            let (inserted_canonical, inserted_squares) = sudoku_base.canonical_to_db();

            thread_sudokus.lock().unwrap().push(inserted_canonical);
            thread_squares.lock().unwrap().extend(inserted_squares);
        }
    });

    loop {
        print!(" {overlap_count}/{total_count} overlapped\r");
        stdout().flush().unwrap();

        let passed_sudokus = {
            let mut temp = sudokus.lock().unwrap();
            let passed = temp.clone();
            temp.clear();
            passed
        };
        let passed_squares = {
            let mut temp = squares.lock().unwrap();
            let passed = temp.clone();
            temp.clear();
            passed
        };

        let total_rows = (passed_sudokus.len() + passed_squares.len()) as u128;

        let inserted_row = database
            .insert_multiple_simple_sudoku_canonical(passed_sudokus, passed_squares)
            .unwrap_or_else(|err| panic!("ERROR COULDN'T INSERT SUDOKU FILLED IN DATABSE: {err}"));

        total_count += total_rows;
        overlap_count += total_rows - inserted_row as u128;
    }
}

fn games() {
    let mut database = Database::connect().unwrap();
    let canonicals = database.get_all_simple_sudoku_canonical().unwrap();

    let games = Arc::new(Mutex::new(Vec::new()));

    let thread_games = Arc::clone(&games);
    thread::spawn(move || {
        let total: u128 = canonicals.len() as u128;
        let count: u128 = 0;
        for canonical in canonicals.into_iter() {
            let mut sudoku = canonical.to_sudoku();
            for difficulty in SudokuDifficulty::iter() {
                print!(" {count}/{total}: difficulty {difficulty}: \r");

                sudoku.randomize();
                let game = sudoku.generate_from(difficulty);
                thread_games.lock().unwrap().push(game.randomized_to_db());
            }
        }
    });

    loop {
        let passed_games = {
            let mut temp = games.lock().unwrap();
            let passed = temp.clone();
            temp.clear();
            passed
        };

        database
            .insert_multiple_simple_sudoku_game(passed_games)
            .unwrap_or_else(|err| panic!("ERROR COULDN'T INSERT SUDOKU FILLED IN DATABSE: {err}"));
    }
}
