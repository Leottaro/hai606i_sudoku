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
    is_canonical: bool,
}

#[derive(PartialEq, Eq, Copy, Clone)]
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

    pub fn iter() -> impl Iterator<Item = CarpetPattern> {
        vec![
            CarpetPattern::Diagonal(1),
            CarpetPattern::Diagonal(2),
            CarpetPattern::Diagonal(3),
            CarpetPattern::Samurai,
            CarpetPattern::Carpet(2),
            CarpetPattern::Carpet(3)
        ].into_iter()
    }
}

impl std::fmt::Display for CarpetPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CarpetPattern::Double => write!(f, "Double"),
            CarpetPattern::Diagonal(n) => write!(f, "Diagonal({n})"),
            CarpetPattern::Samurai => write!(f, "Samurai"),
            CarpetPattern::Carpet(n) => write!(f, "Carpet({n})"),
        }
    }
}
