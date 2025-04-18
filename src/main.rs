#![allow(dead_code)] // no warning due to unused code

use hai606i_sudoku::{
    carpet_sudoku::{CarpetPattern, CarpetSudoku},
    simple_sudoku::SudokuDifficulty,
};

fn main() {
    for difficulty in SudokuDifficulty::iter() {
        for pattern in CarpetPattern::iter() {
            println!("\n\n\n\n{}: {}", pattern, difficulty);
            let carpet = CarpetSudoku::generate_new(3, pattern, difficulty);
            println!("{}", carpet);
        }
    }
}
