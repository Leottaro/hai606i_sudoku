pub mod button;
#[allow(clippy::module_inception)]
pub mod display;

use crate::{
    carpet_sudoku::{CarpetPattern, CarpetSudoku},
    simple_sudoku::SudokuDifficulty,
};
use macroquad::texture::Texture2D;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[cfg(feature = "database")]
use crate::database::Database;

pub type ButtonFunction = Rc<Box<dyn Fn(&mut SudokuDisplay)>>;
pub struct SudokuDisplay {
    max_height: f32,
    max_width: f32,
    scale_factor: f32,
    grid_size: f32,
    pixel_per_cell: f32,
    x_offset: f32,
    y_offset: f32,
    font: macroquad::text::Font,
    button_list: Vec<Button>,
    actions_boutons: HashMap<String, ButtonFunction>,
    background_victoire: Texture2D,
    background_defaite: Texture2D,

    mode: String,
    hovered_cell: Option<(usize, usize, usize)>,
    selected_cell: Option<(usize, usize, usize)>,
    note: bool,
    lifes: usize,
    player_pboard: Vec<Vec<Vec<HashSet<usize>>>>,
    player_pboard_history: Vec<Vec<Vec<Vec<HashSet<usize>>>>>,

    #[cfg(feature = "database")]
    database: Option<Database>,
    carpet: CarpetSudoku,
    difficulty: SudokuDifficulty,
    pattern: CarpetPattern,
    correction_board: Vec<Vec<Vec<usize>>>,
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
