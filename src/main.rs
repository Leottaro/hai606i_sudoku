#![allow(dead_code)] // no warning due to unused code

use hai606i_sudoku::simple_sudoku::{Sudoku, SudokuDifficulty};

fn main() {
    let canonical1 = Sudoku::generate_full(3);
    let (db_canonical_sudoku, _db_canonical_squares) = canonical1.canonical_to_db();
    let canonical2 = Sudoku::db_from_canonical(db_canonical_sudoku);

    if canonical1.ne(&canonical2) {
        panic!("canonical_to_db PROBLEME");
    }

    for difficulty in SudokuDifficulty::iter() {
        let mut randomized1 = canonical1.clone();
        randomized1.randomize();

        let game1 = randomized1.generate_from(difficulty);
        let db_game = game1.randomized_to_db();
        let game2 = Sudoku::db_from_game(db_game);
        if game1.ne(&game2) {
            panic!("randomized_to_db game PROBLEME");
        }

        let db_randomized = randomized1.randomized_to_db();
        let randomized2 = Sudoku::db_from_game(db_randomized);
        if randomized1.ne(&randomized2) {
            panic!("randomized_to_db PROBLEME");
        }
    }
}
