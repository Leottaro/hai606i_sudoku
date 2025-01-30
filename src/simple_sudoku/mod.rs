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

pub trait SudokuRules {
    fn naked_singles(&mut self, debug: bool) -> bool;
    fn hidden_singles(&mut self, debug: bool) -> bool;
    fn naked_pairs(&mut self, debug: bool) -> bool;
    fn naked_triples(&mut self, debug: bool) -> bool;
    fn hidden_pairs(&mut self, debug: bool) -> bool;
    fn hidden_triples(&mut self, debug: bool) -> bool;
    fn naked_quads(&mut self, debug: bool) -> bool;
    fn hidden_quads(&mut self, debug: bool) -> bool;
    fn pointing_pair(&mut self, debug: bool) -> bool;
    fn pointing_triple(&mut self, debug: bool) -> bool;
    fn box_reduction(&mut self, debug: bool) -> bool;
    fn x_wing(&mut self, debug: bool) -> bool;
    fn finned_x_wing(&mut self, debug: bool) -> bool;
    fn sashimi_finned_x_wing(&mut self, debug: bool) -> bool;
    fn franken_x_wing(&mut self, debug: bool) -> bool;
    fn skyscraper(&mut self, debug: bool) -> bool;
    fn y_wing(&mut self, debug: bool) -> bool;
    fn w_wing(&mut self, debug: bool) -> bool;
    fn swordfish(&mut self, debug: bool) -> bool;
    fn finned_swordfish(&mut self, debug: bool) -> bool;
    fn sashimi_finned_swordfish(&mut self, debug: bool) -> bool;
    fn xyz_wing(&mut self, debug: bool) -> bool;
    fn bi_value_universal_grave(&mut self, debug: bool) -> bool;
    fn xy_chain(&mut self, debug: bool) -> bool;
    fn jellyfish(&mut self, debug: bool) -> bool;
    fn finned_jellyfish(&mut self, debug: bool) -> bool;
    fn sashimi_finned_jellyfish(&mut self, debug: bool) -> bool;
    fn wxyz_wing(&mut self, debug: bool) -> bool;
    fn subset_exclusion(&mut self, debug: bool) -> bool;
    fn empty_rectangle(&mut self, debug: bool) -> bool;
    fn almost_locked_set_forcing_chain(&mut self, debug: bool) -> bool;
    fn death_blossom(&mut self, debug: bool) -> bool;
    fn pattern_overlay(&mut self, debug: bool) -> bool;
    fn bowmans_bingo(&mut self, debug: bool) -> bool;
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
