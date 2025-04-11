#![allow(dead_code)] // no warning due to unused code

#[cfg(feature = "database")]
use std::{ sync::mpsc, thread::{ self, sleep }, time::Duration };

#[cfg(feature = "database")]
use hai606i_sudoku::database::Database;

use hai606i_sudoku::{ carpet_sudoku::{ CarpetPattern, CarpetSudoku }, display::SudokuDisplay };

fn main() {
    // env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let mut database = Database::connect().unwrap();

    let carpet1 = CarpetSudoku::generate_full(3, CarpetPattern::Double);
    let (db_carpet, db_carpet_sudokus) = carpet1.db_to_filled().unwrap();
    let (db_sudokus, db_sudokus_squares): (Vec<_>, Vec<_>) = carpet1
        .db_sudokus_to_filled()
        .into_iter()
        .unzip();
    let carpet2 = CarpetSudoku::db_from_filled(
        db_carpet.clone(),
        db_carpet_sudokus.clone(),
        db_sudokus.clone()
    );

    if carpet1.ne(&carpet2) {
        panic!("carpet 1 != carpet2: \nCARPET 1\n{carpet1}\nCARPET 2\n{carpet2}");
    }

    database
        .insert_multiple_canonical_sudokus(
            db_sudokus,
            db_sudokus_squares.into_iter().flatten().collect::<Vec<_>>()
        )
        .unwrap();
    database.insert_canonical_carpet(db_carpet, db_carpet_sudokus).unwrap();

    let carpet3 = database
        .get_random_canonical_carpet(carpet1.get_n() as i16, CarpetPattern::Double.to_db())
        .unwrap();

    if carpet1.ne(&carpet3) {
        panic!("carpet 1 != carpet3: \nCARPET 1\n{carpet1}\nCARPET 3\n{carpet3}");
    }
}
