use crate::simple_sudoku::{Sudoku, SudokuDifficulty};
use std::collections::{HashMap, HashSet};

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
    Simple,
    Double,
    Samurai,
    Diagonal(usize),
    Carpet(usize),
}

pub type RawLink = ((usize, usize), (usize, usize));
impl CarpetPattern {
    pub fn to_db(&self) -> (i16, Option<i16>) {
        match self {
            CarpetPattern::Simple => (0, None),
            CarpetPattern::Double => (1, None),
            CarpetPattern::Samurai => (2, None),
            CarpetPattern::Diagonal(n) => (3, Some(*n as i16)),
            CarpetPattern::Carpet(n) => (4, Some(*n as i16)),
        }
    }

    pub fn from_db(pattern: i16, pattern_size: Option<i16>) -> Self {
        match (pattern, pattern_size) {
            (0, None) => CarpetPattern::Simple,
            (1, None) => CarpetPattern::Double,
            (2, None) => CarpetPattern::Samurai,
            (3, Some(n)) => CarpetPattern::Diagonal(n as usize),
            (4, Some(n)) => CarpetPattern::Carpet(n as usize),
            (a, b) => panic!("pattern:{a} & pattern_size:{:?} not recognized !", b),
        }
    }

    pub fn iter() -> impl Iterator<Item = CarpetPattern> {
        vec![
            CarpetPattern::Simple,
            CarpetPattern::Double,
            CarpetPattern::Diagonal(3),
            CarpetPattern::Diagonal(4),
            CarpetPattern::Diagonal(5),
            CarpetPattern::Samurai,
            CarpetPattern::Carpet(2),
            CarpetPattern::Carpet(3),
        ]
        .into_iter()
    }

    pub fn get_n_sudokus(&self) -> usize {
        match self {
            CarpetPattern::Simple => 1,
            CarpetPattern::Double => 2,
            CarpetPattern::Diagonal(size) => *size,
            CarpetPattern::Samurai => 5,
            CarpetPattern::Carpet(size) => *size * *size,
        }
    }

    pub fn get_raw_links(&self, n: usize) -> Vec<RawLink> {
        let up_left = 0;
        let up_right = n - 1;
        let bottom_left = n * (n - 1);
        let bottom_right = n * n - 1;

        match self {
            CarpetPattern::Simple => vec![],
            CarpetPattern::Double => vec![((0, up_right), (1, bottom_left))],
            CarpetPattern::Diagonal(size) => {
                let size = *size;
                (1..size)
                    .map(|i| ((i - 1, up_right), (i, bottom_left)))
                    .collect()
            }
            CarpetPattern::Samurai => vec![
                ((0, up_left), (1, bottom_right)),
                ((0, up_right), (2, bottom_left)),
                ((0, bottom_left), (3, up_right)),
                ((0, bottom_right), (4, up_left)),
            ],
            CarpetPattern::Carpet(size) => {
                let size = *size;
                let mut links = Vec::new();
                for y in 0..size {
                    for x in 0..size {
                        let sudoku_i = y * size + x;

                        if y < size - 1 {
                            let bottom_i = (y + 1) * size + x;
                            links.extend(
                                (0..n).map(|k| {
                                    ((sudoku_i, bottom_left + k), (bottom_i, up_left + k))
                                }),
                            );
                        }

                        if x < size - 1 {
                            let right_i = y * size + x + 1;
                            links.extend((0..n).map(|k| {
                                ((sudoku_i, n * k + up_right), (right_i, n * k + up_left))
                            }));
                        }

                        if y < size - 1 && x < size - 1 {
                            let corner_i = (y + 1) * size + x + 1;
                            links.push(((sudoku_i, bottom_right), (corner_i, up_left)));
                        }

                        if y < size - 1 && x > 0 {
                            let corner_i = (y + 1) * size + x - 1;
                            links.push(((sudoku_i, bottom_left), (corner_i, up_right)));
                        }
                    }
                }
                links
            }
        }
    }
}

impl std::fmt::Display for CarpetPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CarpetPattern::Simple => write!(f, "Simple"),
            CarpetPattern::Double => write!(f, "Double"),
            CarpetPattern::Diagonal(n) => write!(f, "Diagonal({n})"),
            CarpetPattern::Samurai => write!(f, "Samurai"),
            CarpetPattern::Carpet(n) => write!(f, "Carpet({n})"),
        }
    }
}
