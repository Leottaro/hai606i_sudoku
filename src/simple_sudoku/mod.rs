use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub mod button;
pub mod display;
pub mod rules;
pub mod sudoku;

#[derive(Debug, Clone, Copy)]
pub enum SudokuGroups {
    ROW,
    COLUMN,
    LINES,
    SQUARE,
    ALL,
}

impl PartialEq for SudokuGroups {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (SudokuGroups::ROW, SudokuGroups::ROW)
                | (SudokuGroups::COLUMN, SudokuGroups::COLUMN)
                | (SudokuGroups::LINES, SudokuGroups::LINES)
                | (SudokuGroups::SQUARE, SudokuGroups::SQUARE)
                | (SudokuGroups::ALL, SudokuGroups::ALL)
        )
    }
}

impl Eq for SudokuGroups {}

impl std::hash::Hash for SudokuGroups {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

#[derive(Debug)]
pub struct Sudoku {
    n: usize,
    n2: usize,
    groups: HashMap<SudokuGroups, Vec<HashSet<(usize, usize)>>>,
    cell_groups: HashMap<(usize, usize, SudokuGroups), HashSet<(usize, usize)>>,

    board: Vec<Vec<usize>>,
    possibility_board: Vec<Vec<HashSet<usize>>>,
    difficulty: Option<usize>,
}

pub struct SudokuDisplay<'a> {
    sudoku: &'a mut Sudoku,
    max_scale: f32,
    scale_factor: f32,
    grid_size: f32,
    pixel_per_cell: f32,
    selected_cell: Option<(usize, usize)>,
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
    pub clickable: bool,
    pub text: String,
    pub clicked: bool,
    pub hover: bool,
    pub scale_factor: f32,
}
