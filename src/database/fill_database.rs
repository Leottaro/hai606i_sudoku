use std::{
    io::{stdout, Write},
    ops::SubAssign,
    sync::{Arc, Mutex},
    thread::{self, available_parallelism},
};

use hai606i_sudoku::{
    database::Database,
    simple_sudoku::{Sudoku, SudokuDifficulty},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <canonical|games>", args[0]);
        return;
    }

    if args[1].eq("canonical") {
        if args.len() < 3 {
            eprintln!("Usage: {} canonical <inserted_number>", args[0]);
            return;
        }
        let inserted_number = args[2].parse::<usize>().unwrap();
        canonical(inserted_number);
        return;
    } else if args[1].eq("games") {
        games();
        return;
    }
    eprintln!("Usage: {} <canonical|games>", args[0]);
}

fn canonical(inserted_number: usize) {
    let mut database = Database::connect().unwrap();
    let mut overlap_count: u128 = 0;
    let mut total_count: u128 = 0;

    let remaining_number = Arc::new(Mutex::new(inserted_number));
    let data = Arc::new(Mutex::new(Vec::new()));

    let threads_number = available_parallelism().unwrap();
    let mut thread_handles = Vec::new();
    for thread_id in 0..threads_number.into() {
        let remaining_number = Arc::clone(&remaining_number);
        let thread_data = Arc::clone(&data);
        let handle = thread::Builder::new()
            .name(thread_id.to_string())
            .spawn(move || {
                while *remaining_number.lock().unwrap() > 0 {
                    let sudoku_base: Sudoku = Sudoku::generate_full(3);
                    let inserted_data = sudoku_base.db_to_canonical();
                    thread_data.lock().unwrap().push(inserted_data);
                }
            })
            .unwrap();
        thread_handles.push(handle);
    }

    while *remaining_number.lock().unwrap() > 0 {
        print!(
            " {} sudoku remaining: {overlap_count}/{total_count} rows overlapped\r",
            remaining_number.lock().unwrap()
        );
        stdout().flush().unwrap();

        let (passed_sudokus, passed_squares) = {
            let mut locked = data.lock().unwrap();
            let drain_number = remaining_number.lock().unwrap().min(locked.len());
            let (sudokus, squares): (Vec<_>, Vec<_>) = locked.drain(0..drain_number).unzip();
            (sudokus, squares.into_iter().flatten().collect::<Vec<_>>())
        };

        let total_rows = passed_sudokus.len() + passed_squares.len();
        let (just_inserted_sudokus, just_inserted_squares) = database
            .insert_multiple_simple_sudoku_canonical(passed_sudokus, passed_squares)
            .unwrap_or_else(|err| panic!("ERROR COULDN'T INSERT SUDOKU FILLED IN DATABSE: {err}"));

        remaining_number
            .lock()
            .unwrap()
            .sub_assign(just_inserted_sudokus);
        total_count += total_rows as u128;
        overlap_count += (total_rows - just_inserted_sudokus - just_inserted_squares) as u128;
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}

fn games() {
    let database = Arc::new(Mutex::new(Database::connect().unwrap()));

    let (mut remaining_canonicals, canonicals) = {
        let temp = database
            .lock()
            .unwrap()
            .get_all_simple_sudoku_canonical()
            .unwrap();
        (temp.len(), temp.into_iter())
    };

    for canonical in canonicals {
        remaining_canonicals.sub_assign(1);
        let mut sudoku = Sudoku::db_from_canonical(canonical);
        let mut passed_games = Vec::new();
        println!(
            "{} canonicals left:{}",
            remaining_canonicals,
            " ".repeat(50)
        );

        for difficulty in SudokuDifficulty::iter() {
            println!("{difficulty}{}", " ".repeat(50));

            sudoku.randomize();
            let game = sudoku.generate_from(difficulty);
            let mut game_db = game.db_to_randomized();
            game_db.game_difficulty = difficulty as u8;
            passed_games.push(game_db);
        }

        let thread_database = Arc::clone(&database);
        thread::spawn(move || {
            thread_database
                .lock()
                .unwrap()
                .insert_multiple_simple_sudoku_game(passed_games)
                .unwrap_or_else(|err| {
                    panic!("ERROR COULDN'T INSERT SUDOKU GAME IN DATABSE: {err}")
                });
        });
    }
}
