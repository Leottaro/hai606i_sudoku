use std::{
    sync::{Arc, Mutex},
    thread,
};

use hai606i_sudoku::{
    database::{DBNewSimpleSudoku, Database},
    simple_sudoku::{Sudoku, SudokuDifficulty},
};

fn main() {
    let database = Arc::new(Mutex::new(Database::connect().unwrap()));
    let difficulties = SudokuDifficulty::iter().collect::<Vec<_>>();
    for (count, &difficulty) in (0_u128..).zip(difficulties.iter().cycle()) {
        println!("\n{count}: difficulty {difficulty}: ");

        let sudoku_base = Sudoku::generate_full(3);
        let sudoku = Sudoku::generate_new(3, difficulty, Some(sudoku_base));

        let thread_database = Arc::clone(&database);
        let mut inserted_sudoku = DBNewSimpleSudoku::from(&sudoku);
        inserted_sudoku.difficulty = difficulty as u8;
        thread::spawn(move || {
            thread_database
                .lock()
                .unwrap()
                .insert_simple_sudoku(inserted_sudoku)
                .unwrap_or_else(|err| {
                    panic!("ERROR COULDN'T INSERT GENERATED SUDOKU IN DATABSE: {err}")
                });
        });
    }
}
