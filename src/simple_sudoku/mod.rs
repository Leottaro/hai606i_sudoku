use std::collections::HashSet;

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
    selected_buttons: HashSet<(usize, usize)>,
    x_offset: f32,
    y_offset: f32,
    bx_offset: f32,
    solvex_offset: f32,
    choosey_offset: f32,
    mode: String,
    solving: bool,
    player_pboard: Vec<Vec<HashSet<usize>>>,
    note: bool,
}
