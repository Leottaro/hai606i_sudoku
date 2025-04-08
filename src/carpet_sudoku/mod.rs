use std::collections::{ HashMap, HashSet };
use crate::simple_sudoku::{ Sudoku, SudokuDifficulty };

pub mod carpet;

pub type CarpetLink = ((usize, usize), (usize, usize)); // (usize, usize) == (sudoku_i, square_i)
#[derive(Clone)]
pub struct CarpetSudoku {
    n: usize,
    n2: usize,
    sudokus: Vec<Sudoku>,
    links: HashMap<usize, HashSet<(usize, usize, usize)>>,
    difficulty: SudokuDifficulty,

    is_canonical: bool,
}

pub enum CarpetPattern {
    Double,
    Diagonal(usize),
    Samurai,
    Carpet(usize),
}
