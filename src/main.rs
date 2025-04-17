#![allow(dead_code)] // no warning due to unused code

// use hai606i_sudoku::database::Database;
use hai606i_sudoku::carpet_sudoku::{CarpetPattern, CarpetSudoku};

fn main() {
    // let mut database = Database::connect().unwrap();
    // println!("connecté");

    for pattern in CarpetPattern::iter() {
        println!("\n\n\n\n\nPATTERN: {pattern}");
        let carpet1 = CarpetSudoku::generate_full(3, pattern);
        println!("carpet1 généré: \n{carpet1}");
        let (db_carpet, db_carpet_sudokus) = carpet1.db_to_filled().unwrap();
        let (db_sudokus, _db_sudokus_squares): (Vec<_>, Vec<_>) =
            carpet1.db_sudokus_to_filled().into_iter().unzip();
        println!("carpet1 database généré");

        let carpet2 = CarpetSudoku::db_from_filled(
            db_carpet.clone(),
            db_carpet_sudokus.clone(),
            db_sudokus.clone(),
        );
        println!("carpet2 reconstruit: \n{carpet2}");

        if carpet1.ne(&carpet2) {
            panic!("carpet1 != carpet2");
        }

        // database
        //     .insert_multiple_canonical_sudokus(
        //         db_sudokus,
        //         db_sudokus_squares.into_iter().flatten().collect::<Vec<_>>()
        //     )
        //     .unwrap();
        // println!("carpet sudokus inserés");

        // database.insert_canonical_carpet(db_carpet, db_carpet_sudokus).unwrap();
        // println!("carpet inseré");
    }
}
