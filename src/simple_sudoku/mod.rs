pub type Coords = (usize, usize);

use std::collections::{HashMap, HashSet};

pub mod rules;
pub mod sudoku;
pub mod sudoku_generation;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum SudokuGroups {
    Row = 0,
    Column = 1,
    Lines = 2,
    Square = 3,
    All = 4,
}

impl std::fmt::Display for SudokuGroups {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuGroups::Row => write!(f, "ROW"),
            SudokuGroups::Column => write!(f, "COLUMN"),
            SudokuGroups::Lines => write!(f, "LINES"),
            SudokuGroups::Square => write!(f, "SQUARE"),
            SudokuGroups::All => write!(f, "ALL"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum SudokuDifficulty {
    Unknown = 0,
    Easy = 1,
    Medium = 2,
    Hard = 3,
    Master = 4,
    Extreme = 5,
    Useless = 254,
    Unimplemented = 255,
}

impl std::fmt::Display for SudokuDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuDifficulty::Unknown => write!(f, "Unknown"),
            SudokuDifficulty::Easy => write!(f, "Easy"),
            SudokuDifficulty::Medium => write!(f, "Medium"),
            SudokuDifficulty::Hard => write!(f, "Hard"),
            SudokuDifficulty::Master => write!(f, "Master"),
            SudokuDifficulty::Extreme => write!(f, "Extreme"),
            SudokuDifficulty::Useless => write!(f, "Useless"),
            SudokuDifficulty::Unimplemented => write!(f, "Unimplemented"),
        }
    }
}

impl SudokuDifficulty {
    pub fn iter() -> impl Iterator<Item = SudokuDifficulty> {
        vec![
            SudokuDifficulty::Easy,
            SudokuDifficulty::Medium,
            SudokuDifficulty::Hard,
            SudokuDifficulty::Master,
            SudokuDifficulty::Extreme,
        ]
        .into_iter()
    }

    pub fn from(n: i16) -> Self {
        match n {
            1 => SudokuDifficulty::Easy,
            2 => SudokuDifficulty::Medium,
            3 => SudokuDifficulty::Hard,
            4 => SudokuDifficulty::Master,
            5 => SudokuDifficulty::Extreme,
            254 => SudokuDifficulty::Useless,
            255 => SudokuDifficulty::Unimplemented,
            _ => SudokuDifficulty::Unknown,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SudokuDifficulty::Medium => SudokuDifficulty::Easy,
            SudokuDifficulty::Hard => SudokuDifficulty::Medium,
            SudokuDifficulty::Master => SudokuDifficulty::Hard,
            SudokuDifficulty::Extreme => SudokuDifficulty::Master,

            _ => SudokuDifficulty::Unknown,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SudokuDifficulty::Easy => SudokuDifficulty::Medium,
            SudokuDifficulty::Medium => SudokuDifficulty::Hard,
            SudokuDifficulty::Hard => SudokuDifficulty::Master,
            SudokuDifficulty::Master => SudokuDifficulty::Extreme,

            _ => SudokuDifficulty::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SudokuError {
    CanonizationMismatch(Box<Sudoku>, u64),
    InvalidState(String),
    NoPossibilityCell(Coords),
    ParseString((String, String)),
    ReadFile((String, String)),
    SameValueCells((Coords, Coords)),
    WrongFunction(String),
    WrongInput(String),
}

impl std::fmt::Display for SudokuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuError::CanonizationMismatch(sudoku, board_hash) => {
                write!(
                    f,
                    "SudokuError: Canonization mismatch this sudoku \n{sudoku}\nExpected board hash {}, got {}",
                    sudoku.canonical_filled_board_hash,
                    board_hash
                )
            }
            SudokuError::InvalidState(string) => {
                write!(f, "SudokuError: Invalid sudoku state for {string}")
            }
            SudokuError::NoPossibilityCell((x, y)) => {
                write!(f, "SudokuError: No possibility for cell at ({x},{y})")
            }
            SudokuError::ParseString((string, error)) => {
                write!(f, "SudokuError: couldn't parse \"{string}\": {error}")
            }
            SudokuError::ReadFile((file_path, error)) => {
                write!(f, "SudokuError: couldn't open file {file_path}: {error}")
            }
            SudokuError::SameValueCells(((x1, y1), (x2, y2))) => {
                write!(
                    f,
                    "SudokuError: Cells at ({x1},{y1}) and ({x2},{y2}) have the same value"
                )
            }
            SudokuError::WrongFunction(string) => {
                write!(f, "SudokuError: Wrong function for {string}")
            }
            SudokuError::WrongInput(string) => {
                write!(f, "SudokuError: Wrong input for {string}")
            }
        }
    }
}

pub type SudokuRule = fn(&mut Sudoku) -> Result<bool, SudokuError>;
type GroupMap = HashMap<SudokuGroups, Vec<HashSet<Coords>>>;
type CellGroupMap = HashMap<(Coords, SudokuGroups), HashSet<Coords>>;

#[derive(Debug, Clone)]
pub struct Sudoku {
    n: usize,
    n2: usize,
    board: Vec<Vec<usize>>,
    possibility_board: Vec<Vec<HashSet<usize>>>,
    filled_cells: usize,
    difficulty: SudokuDifficulty,

    is_canonical: bool,
    canonical_filled_board_hash: u64,
    values_swap: HashMap<usize, Coords>, // 1 -> (2, 3) exprime les règles 1 donne 2 et 3 donne 1
    rows_swap: HashMap<usize, Coords>,
}
