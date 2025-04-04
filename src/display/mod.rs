pub mod button;
pub mod display;

use std::{ collections::{ HashMap, HashSet }, rc::Rc };
use macroquad::texture::Texture2D;
use crate::{ carpet_sudoku::CarpetSudoku, simple_sudoku::SudokuDifficulty };

#[cfg(feature = "database")]
use crate::database::Database;

pub type ButtonFunction = Rc<Box<dyn Fn(&mut SudokuDisplay)>>;
pub struct SudokuDisplay {
    #[cfg(feature = "database")]
    database: Option<Database>,
    carpet: CarpetSudoku,
    max_height: f32,
    max_width: f32,
    scale_factor: f32,
    grid_size: f32,
    pixel_per_cell: f32,
    selected_cell: Option<(usize, usize, usize)>,
    x_offset: f32,
    y_offset: f32,
    mode: String,
    player_pboard_history: Vec<Vec<Vec<Vec<HashSet<usize>>>>>,
    player_pboard: Vec<Vec<Vec<HashSet<usize>>>>,
    note: bool,
    button_list: Vec<Button>,
    font: macroquad::text::Font,
    actions_boutons: HashMap<String, ButtonFunction>,
    background_victoire: Texture2D,
    background_defaite: Texture2D,
    lifes: usize,
    new_game_available: bool,
    difficulty: SudokuDifficulty,
    correction_board: Vec<Vec<usize>>,
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
