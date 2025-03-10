pub type Coords = (usize, usize);

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

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
    Unknown = u8::MIN as isize,
    Mandatory = 1,
    Easy = 2,
    Medium = 3,
    Hard = 4,
    Master = 5,
    Extreme = 6,
    Unimplemented = u8::MAX as isize,
}

impl std::fmt::Display for SudokuDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuDifficulty::Mandatory => write!(f, "MANDATORY"),
            SudokuDifficulty::Unknown => write!(f, "UNKNOWN"),
            SudokuDifficulty::Easy => write!(f, "EASY"),
            SudokuDifficulty::Medium => write!(f, "MEDIUM"),
            SudokuDifficulty::Hard => write!(f, "HARD"),
            SudokuDifficulty::Master => write!(f, "MASTER"),
            SudokuDifficulty::Extreme => write!(f, "EXTREME"),
            SudokuDifficulty::Unimplemented => write!(f, "UNIMPLEMENTED"),
        }
    }
}

#[allow(dead_code)]
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
}

pub type SudokuRule = fn(&mut Sudoku) -> bool;
type GroupMap = HashMap<SudokuGroups, Vec<HashSet<Coords>>>;
type CellGroupMap = HashMap<(Coords, SudokuGroups), HashSet<Coords>>;

#[derive(Debug)]
pub struct Sudoku {
    n: usize,
    n2: usize,

    board: Vec<Vec<usize>>,
    possibility_board: Vec<Vec<HashSet<usize>>>,
    difficulty: SudokuDifficulty,
    error: Option<(Coords, Coords)>,
}

pub type ButtonFunction = Rc<Box<dyn Fn(&mut SudokuDisplay)>>;
pub struct SudokuDisplay<'a> {
    sudoku: &'a mut Sudoku,
    max_scale: f32,
    scale_factor: f32,
    grid_size: f32,
    pixel_per_cell: f32,
    selected_cell: Option<Coords>,
    x_offset: f32,
    y_offset: f32,
    bx_offset: f32,
    solvex_offset: f32,
    mode: String,
    player_pboard: Vec<Vec<HashSet<usize>>>,
    note: bool,
    button_list: Vec<Button>,
    font: macroquad::text::Font,
    actions_boutons: HashMap<String, ButtonFunction>,
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
