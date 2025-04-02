#![allow(dead_code)] // no warning due to unused code

use hai606i_sudoku::{
    carpet_sudoku::{ CarpetPattern, CarpetSudoku },
    simple_sudoku::SudokuDifficulty,
};

fn main() {
    let mut count = 0;
    loop {
        count += 1;
        println!("count: {count}{}", " ".repeat(50));
        CarpetSudoku::generate_new(3, CarpetPattern::Double, SudokuDifficulty::Extreme);
        // Sudoku::generate_new(3, SudokuDifficulty::Medium);
    }
}
