use std::{collections::HashSet, rc::Rc};

pub mod button;
pub mod display;
pub mod rules;
pub mod sudoku;

#[derive(Debug)]
pub struct Sudoku {
    n: usize,
    n2: usize,
    board: Vec<Vec<usize>>,
    possibility_board: Vec<Vec<HashSet<usize>>>,
}

pub struct SudokuDisplay<'a> {
    sudoku: &'a mut Sudoku,
    max_scale: f32,
    scale_factor: f32,
    grid_size: f32,
    pixel_per_cell: f32,
    selected_cell: Option<(usize, usize)>,
    selected_buttons: HashSet<usize>,
    x_offset: f32,
    y_offset: f32,
    bx_offset: f32,
    solvex_offset: f32,
    mode: String,
    player_pboard: Vec<Vec<HashSet<usize>>>,
    note: bool,
    button_list: Vec<Button>,
    font: macroquad::text::Font,
    actions_boutons: std::collections::HashMap<String, Rc<Box<dyn Fn(&mut SudokuDisplay)>>>,
}

pub struct Button {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub enabled: bool,
    pub text: String,
    pub clicked: bool,
    pub hover: bool,
    pub scale_factor: f32,
}
