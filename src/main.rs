#![allow(dead_code)] // no warning due to unused code

#[cfg(feature = "database")]
use hai606i_sudoku::database::Database;

use hai606i_sudoku::carpet_sudoku::{ CarpetPattern, CarpetSudoku };

fn main() {
    // env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let mut database = Database::connect().unwrap();
    println!("connecté");

    let carpet1 = CarpetSudoku::generate_full(3, CarpetPattern::Double);
    println!("carpet1 généré: \n{carpet1}");

    let (db_carpet, db_carpet_sudokus) = carpet1.db_to_filled().unwrap();
    let (db_sudokus, db_sudokus_squares): (Vec<_>, Vec<_>) = carpet1
        .db_sudokus_to_filled()
        .into_iter()
        .unzip();
    println!("carpet1 database généré");

    let carpet2 = CarpetSudoku::db_from_filled(
        db_carpet.clone(),
        db_carpet_sudokus.clone(),
        db_sudokus.clone()
    );
    println!("carpet2 reconstruit: \n{carpet2}");

    if carpet1.ne(&carpet2) {
        panic!("carpet 1 != carpet2");
    }

    database
        .insert_multiple_canonical_sudokus(
            db_sudokus,
            db_sudokus_squares.into_iter().flatten().collect::<Vec<_>>()
        )
        .unwrap();
    println!("carpet sudokus inserés");

    database.insert_canonical_carpet(db_carpet, db_carpet_sudokus).unwrap();
    println!("carpet inseré");

    let carpet3 = database
        .get_random_canonical_carpet(carpet1.get_n() as i16, CarpetPattern::Double.to_db())
        .unwrap();
    println!("carpet3 reconstruit: \n{carpet3}");

    if carpet1.ne(&carpet3) {
        panic!("carpet 1 != carpet3");
    }
}
