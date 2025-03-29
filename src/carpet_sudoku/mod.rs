use std::collections::HashMap;

use crate::simple_sudoku::{Sudoku, SudokuDifficulty};

pub mod sudoku;

pub type CarpetLink = ((usize, usize), (usize, usize)); // (usize, usize) == (sudoku_i, square_i)
#[derive(Clone)]
pub struct CarpetSudoku {
    n: usize,
    n2: usize,
    sudokus: Vec<Sudoku>,
    links: HashMap<usize, Vec<(usize, usize, usize)>>,
    filled_cells: usize,
    difficulty: SudokuDifficulty,
}

pub enum CarpetPattern {
    Double,
    Diagonal(usize),
    Samurai,
}
