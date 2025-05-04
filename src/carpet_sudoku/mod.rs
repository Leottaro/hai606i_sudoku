use crate::simple_sudoku::{Coords, Sudoku, SudokuDifficulty};
use std::collections::{HashMap, HashSet};

pub mod carpet;
mod carpet_generation;
pub mod pattern;

pub type CarpetLinks = HashMap<usize, HashSet<(usize, usize, usize)>>;
type RawLink = (Coords, Coords);
#[derive(Clone)]
pub struct CarpetSudoku {
    n: usize,
    n2: usize,
    pattern: CarpetPattern,
    sudokus: Vec<Sudoku>,
    links: CarpetLinks,

    difficulty: SudokuDifficulty,
    difficulty_score: usize,

    filled_board_hash: u64,
    is_canonical: bool,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum CarpetPattern {
    Simple,
    Samurai,
    Diagonal(usize),
    DenseDiagonal(usize),
    Carpet(usize),
    DenseCarpet(usize),
    Thorus(usize),
    DenseThorus(usize),
    Custom(usize),
}
