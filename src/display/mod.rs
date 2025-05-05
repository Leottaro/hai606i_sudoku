pub mod button;
#[allow(clippy::module_inception)]
pub mod display;

use crate::{
    carpet_sudoku::{CarpetPattern, CarpetSudoku},
    simple_sudoku::{Coords, SudokuDifficulty},
};

use macroquad::texture::Texture2D;
use macroquad::color::Color;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Instant,
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
    last_processed_keys: Option<Instant>,

    mode: String,
    analyse_text: Vec<String>,
    hovered_cell: Option<(usize, usize, usize)>,
    selected_cell: Option<(usize, usize, usize)>,
    note: bool,
    lifes: usize,
    #[allow(clippy::type_complexity)]
    wrong_cell: Arc<Mutex<Option<(usize, usize, usize, usize)>>>,
    wrong_cell_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    player_pboard: Vec<Vec<Vec<HashMap<usize, u32>>>>,
    #[allow(clippy::type_complexity)]
    player_pboard_history: Vec<Vec<Vec<Vec<HashMap<usize, u32>>>>>,
    selected_color: u32,
    pattern_list: Vec<CarpetPattern>,
    thorus_view: Coords,

    #[cfg(feature = "database")]
    database: Option<Database>,
    carpet: CarpetSudoku,
    difficulty: SudokuDifficulty,
    pattern: CarpetPattern,
    correction_board: Vec<Vec<Vec<usize>>>,
    cloud_texture: Texture2D,
    no_cloud_texture: Texture2D,
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
    pub background_color: Color,
    pub draw_text: bool,
    pub draw_border: bool,
    pub stroke: bool
}
