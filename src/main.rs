#![allow(dead_code)] // no warning due to unused code

use hai606i_sudoku::{
    carpet_sudoku::{CarpetPattern, CarpetSudoku},
    simple_sudoku::SudokuDifficulty,
};

fn main() {
    let original_carpet = CarpetSudoku::generate_full(3, CarpetPattern::Double);
    let generated_carpet = original_carpet.generate_from(SudokuDifficulty::Easy);
    println!("{generated_carpet}");
}
