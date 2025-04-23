use crate::simple_sudoku::{Sudoku, SudokuDifficulty};
use std::collections::{HashMap, HashSet};

pub mod carpet;
mod carpet_generation;
pub mod pattern;

pub type CarpetLinks = HashMap<usize, HashSet<(usize, usize, usize)>>;
type RawLink = ((usize, usize), (usize, usize));
#[derive(Clone)]
pub struct CarpetSudoku {
    n: usize,
    n2: usize,
    pattern: CarpetPattern,
    sudokus: Vec<Sudoku>,
    links: CarpetLinks,
    difficulty: SudokuDifficulty,

    filled_board_hash: u64,
    is_canonical: bool,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum CarpetPattern {
    Simple,
    Double,
    Samurai,
    Diagonal(usize),
    Carpet(usize),
    Custom(usize),
}
