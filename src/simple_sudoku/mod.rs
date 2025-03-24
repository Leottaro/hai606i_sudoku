pub type Coords = (u8, u8);

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use macroquad::texture::Texture2D;

use crate::database::Database;

pub mod button;
pub mod display;
pub mod rules;
pub mod sudoku;

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
            SudokuDifficulty::Unknown => write!(f, "UNKNOWN"),
            SudokuDifficulty::Easy => write!(f, "EASY"),
            SudokuDifficulty::Medium => write!(f, "MEDIUM"),
            SudokuDifficulty::Hard => write!(f, "HARD"),
            SudokuDifficulty::Master => write!(f, "MASTER"),
            SudokuDifficulty::Extreme => write!(f, "EXTREME"),
            SudokuDifficulty::Useless => write!(f, "USELESS"),
            SudokuDifficulty::Unimplemented => write!(f, "UNIMPLEMENTED"),
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

    pub fn from(n: u8) -> Self {
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
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum SudokuError {
    NoPossibilityCell(Coords),
    SameValueCells((Coords, Coords)),
}

impl std::fmt::Display for SudokuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuError::NoPossibilityCell(coords) => {
                write!(f, "SudokuError: No possibility for cell at {:?}", coords)
            }
            SudokuError::SameValueCells((coords1, coords2)) => {
                write!(
                    f,
                    "SudokuError: Cells at {:?} and {:?} have the same value",
                    coords1, coords2
                )
            }
        }
    }
}

pub type SudokuRule = fn(&mut Sudoku) -> bool;
type GroupMap = HashMap<SudokuGroups, Vec<HashSet<Coords>>>;
type CellGroupMap = HashMap<(Coords, SudokuGroups), HashSet<Coords>>;

#[derive(Clone)]
pub struct Sudoku {
    n: u8,
    n2: u8,

    board: Vec<Vec<u8>>,
    possibility_board: Vec<Vec<HashSet<u8>>>,
    filled_cells: u16,

    difficulty: SudokuDifficulty,
    error: Option<SudokuError>,

    is_canonical: bool,
    canonical_board_hash: u64,
}

pub type ButtonFunction = Rc<Box<dyn Fn(&mut SudokuDisplay)>>;
pub struct SudokuDisplay {
    database: Option<Database>,
    sudoku: Sudoku,
    max_height: f32,
    max_width: f32,
    scale_factor: f32,
    grid_size: f32,
    pixel_per_cell: f32,
    selected_cell: Option<Coords>,
    x_offset: f32,
    y_offset: f32,
    mode: String,
    player_pboard_history: Vec<Vec<Vec<HashSet<u8>>>>,
    player_pboard: Vec<Vec<HashSet<u8>>>,
    note: bool,
    button_list: Vec<Button>,
    font: macroquad::text::Font,
    actions_boutons: HashMap<String, ButtonFunction>,
    background: Texture2D,
    background_defaite: Texture2D,
    lifes: u8,
    new_game_available: bool,
    difficulty: SudokuDifficulty,
    correction_board: Vec<Vec<u8>>,
}

pub struct Button {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub enabled: bool,
    pub clickable: bool,
    pub text: String,
    pub clicked: bool,
    pub hover: bool,
    pub scale_factor: f32,
}
