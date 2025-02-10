use std::collections::{HashMap, HashSet};

pub mod display;
pub mod rules;
pub mod sudoku;

#[derive(Debug)]
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
    board: Vec<Vec<usize>>,
    possibility_board: Vec<Vec<HashSet<usize>>>,
    groups: HashMap<SudokuGroups, Vec<HashSet<(usize, usize)>>>,
    cell_groups: HashMap<(usize, usize, SudokuGroups), HashSet<(usize, usize)>>,
}

pub struct SudokuDisplay<'a> {
    sudoku: &'a mut Sudoku,
    window_size: f32,
    pixel_per_cell: f32,
    selected_cell: Option<(usize, usize)>,
    selected_buttons: HashSet<(usize, usize)>,
    x_offset: f32,
    y_offset: f32,
}
