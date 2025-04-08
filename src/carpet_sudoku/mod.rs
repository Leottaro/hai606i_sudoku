use std::collections::{ HashMap, HashSet };

use crate::simple_sudoku::{ Sudoku, SudokuDifficulty };

pub mod carpet;

pub type CarpetLink = ((usize, usize), (usize, usize)); // (usize, usize) == (sudoku_i, square_i)
#[derive(Clone)]
pub struct CarpetSudoku {
    n: usize,
    n2: usize,
    pattern: CarpetPattern,
    sudokus: Vec<Sudoku>,
    links: HashMap<usize, HashSet<(usize, usize, usize)>>,
    difficulty: SudokuDifficulty,

    filled_board_hash: u64,
}

#[derive(Clone)]
pub enum CarpetPattern {
    Double,
    Diagonal(usize),
    Samurai,
    Carpet(usize),
}

impl CarpetPattern {
    pub fn to_db(&self) -> (i16, Option<i16>) {
        match self {
            CarpetPattern::Double => (1, Some(2)),
            CarpetPattern::Diagonal(n) => (1, Some(*n as i16)),
            CarpetPattern::Samurai => (2, None),
            CarpetPattern::Carpet(n) => (3, Some(*n as i16)),
        }
    }

    pub fn from_db(pattern: i16, pattern_size: Option<i16>) -> Self {
        match (pattern, pattern_size) {
            (1, Some(2)) => CarpetPattern::Double,
            (1, Some(n)) => CarpetPattern::Diagonal(n as usize),
            (2, None) => CarpetPattern::Samurai,
            (3, Some(n)) => CarpetPattern::Carpet(n as usize),
            (a, b) => panic!("pattern:{a} & pattern_size:{:?} not recognized !", b),
        }
    }
}
