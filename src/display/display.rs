use crate::carpet_sudoku::{CarpetPattern, CarpetSudoku};
#[cfg(feature = "database")]
use crate::database::Database;
use crate::simple_sudoku::{Coords, Sudoku, SudokuDifficulty, SudokuGroups::*};

use super::{Button, ButtonFunction, SudokuDisplay};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PLAY: &str = "Play";
const ANALYSE: &str = "Analyse";
const SOLVE_ONCE: &str = "Solve once";
const SOLVE: &str = "Solve";

const NOTE: &str = "Note";
const FILL_NOTES: &str = "Fill Notes";
const UNDO: &str = "Undo";

const NEW_GAME: &str = "New Game";
const CANCEL: &str = "Cancel";
const EMPTY: &str = "Empty";
const INCREASE: &str = "+";
const DECREASE: &str = "-";
const CREATE: &str = "Create";
const BROWSE: &str = "Browse";

impl SudokuDisplay {
    pub async fn new(n: usize, font: Font) -> Self {
        let max_height = screen_height() * 1.05;
        let max_width = screen_width() * 1.05;
        let scale_factor = 1.0;
        let grid_size = 900.0 * scale_factor;
        let pixel_per_cell = grid_size / ((n * n) as f32);
        let x_offset = 250.0 * scale_factor;
        let y_offset = 150.0 * scale_factor;

        let carpet = CarpetSudoku::new(n, CarpetPattern::Simple);
        let mode = PLAY.to_string();
        let analyse_text = vec!["Ready to analyze".to_string()];
        let player_pboard_history = Vec::new();
        let player_pboard = vec![
            vec![vec![HashSet::new(); carpet.get_n2()]; carpet.get_n2()];
            carpet.get_n_sudokus()
        ];
        let correction_board =
            vec![vec![vec![0; carpet.get_n2()]; carpet.get_n2()]; carpet.get_n_sudokus()];
        let note = false;
        let mut button_list = Vec::new();
        let mut actions_boutons: HashMap<String, ButtonFunction> = HashMap::new();
        let background_victoire = load_texture("./res/bg/bg-petit.png").await.unwrap();
        let background_defaite = load_texture("./res/bg/bg-def.png").await.unwrap();
        let lifes = 3;
        let wrong_cell = Arc::new(Mutex::new(None));
        let wrong_cell_handle = Arc::new(Mutex::new(None));
        let difficulty = SudokuDifficulty::Easy;
        let pattern: CarpetPattern = CarpetPattern::Simple;
        let pattern_list = CarpetPattern::iter_simple().collect::<Vec<_>>();
        let thorus_view = (0, 0);

        // ================== Buttons ==================
        let button_sizex = 150.0 * scale_factor;
        let button_sizey = 100.0 * scale_factor;
        let b_padding = 10.0;

        let bouton_play = Button::new(
            2.0 * b_padding,
            2.0 * b_padding,
            button_sizex,
            button_sizey,
            PLAY.to_string(),
            true,
            scale_factor,
        );
        actions_boutons.insert(
            bouton_play.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode(PLAY);
            })),
        );
        button_list.push(bouton_play);

        let button_analyse = Button::new(
            2.0 * b_padding + b_padding + button_sizex,
            2.0 * b_padding,
            button_sizex,
            button_sizey,
            ANALYSE.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_analyse.text.clone(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode(ANALYSE);
            })),
        );
        button_list.push(button_analyse);

        let game_button_sizex = x_offset - b_padding * 4.;
        let game_button_sizey = button_sizey * 0.9;
        let game_button_offsety = 4.0 * b_padding + button_sizey;

        let new_game_btn = Button::new(
            2.0 * b_padding,
            game_button_offsety,
            game_button_sizex,
            game_button_sizey,
            NEW_GAME.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            new_game_btn.text.to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.set_new_game_btn(false);
                sudoku_display.set_pattern_btn(true);
            })),
        );
        button_list.push(new_game_btn);

        let mut new_game_cancel_btn = Button::new(
            2.0 * b_padding,
            game_button_offsety,
            game_button_sizex,
            game_button_sizey,
            CANCEL.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            new_game_cancel_btn.text.to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.set_new_game_btn(true);

                sudoku_display.set_pattern_btn(false);
                sudoku_display.set_difficulty_btn(false);
                sudoku_display.set_mode_btn(false);
            })),
        );
        new_game_cancel_btn.set_enabled(false);
        button_list.push(new_game_cancel_btn);

        // ==========================================================
        // ================= Sudoku Pattern Buttons =================
        // ==========================================================

        for (i, &pattern) in pattern_list.iter().enumerate() {
            let mut bouton = Button::new(
                2.0 * b_padding,
                game_button_offsety + ((i + 1) as f32) * (b_padding + game_button_sizey),
                game_button_sizex,
                game_button_sizey,
                pattern.to_string(),
                false,
                scale_factor,
            );
            bouton.set_enabled(false);
            button_list.push(bouton);
            actions_boutons.insert(
                pattern.to_string(),
                Rc::new(Box::new(move |sudoku_display| {
                    sudoku_display.pattern = pattern;
                    sudoku_display.set_pattern_btn(false);
                    sudoku_display.set_difficulty_btn(true);
                })),
            );
        }

        // ==========================================================
        // ================== Increase / Decrease ===================
        // ==========================================================
        let decrease_string = DECREASE.to_string();
        let mut decrease_button = Button::new(
            2.0 * b_padding,
            game_button_offsety
                + ((pattern_list.len() + 1) as f32) * (b_padding + game_button_sizey),
            game_button_sizex / 2.,
            game_button_sizey / 3.,
            decrease_string.clone(),
            false,
            scale_factor,
        );
        decrease_button.set_enabled(false);
        button_list.push(decrease_button);
        actions_boutons.insert(
            decrease_string,
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.pattern_size_btn(false);
            })),
        );

        let increase_string = INCREASE.to_string();
        let mut increase_button = Button::new(
            2.0 * b_padding + game_button_sizex / 2.,
            game_button_offsety
                + ((pattern_list.len() + 1) as f32) * (b_padding + game_button_sizey),
            game_button_sizex / 2.,
            game_button_sizey / 3.,
            increase_string.clone(),
            false,
            scale_factor,
        );
        increase_button.set_enabled(false);
        button_list.push(increase_button);
        actions_boutons.insert(
            increase_string,
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.pattern_size_btn(true);
            })),
        );

        // ==========================================================
        // ================== Difficulty Buttons ====================
        // ==========================================================
        for (i, difficulty) in SudokuDifficulty::iter().enumerate() {
            let diff_string = difficulty.to_string();
            let mut bouton = Button::new(
                2.0 * b_padding,
                game_button_offsety + ((i + 2) as f32) * (b_padding + game_button_sizey),
                game_button_sizex,
                game_button_sizey,
                diff_string.clone(),
                false,
                scale_factor,
            );
            bouton.set_enabled(false);
            button_list.push(bouton);
            actions_boutons.insert(
                diff_string,
                Rc::new(Box::new(move |sudoku_display| {
                    sudoku_display.difficulty = difficulty;
                    sudoku_display.set_difficulty_btn(false);
                    sudoku_display.set_mode_btn(true);
                })),
            );
        }

        // ==========================================================
        // ===================== Empty Button =======================
        // ==========================================================
        let mut bouton = Button::new(
            2.0 * b_padding,
            game_button_offsety + b_padding + game_button_sizey,
            game_button_sizex,
            game_button_sizey,
            EMPTY.to_string(),
            false,
            scale_factor,
        );
        bouton.set_enabled(false);
        button_list.push(bouton);
        actions_boutons.insert(
            EMPTY.to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.difficulty = SudokuDifficulty::Unknown;
                sudoku_display.set_difficulty_btn(false);
                sudoku_display.set_mode_btn(false);
                sudoku_display.set_new_game_btn(true);
                sudoku_display.new_game(true, false);
            })),
        );

        // ==========================================================
        // ================== Create and Browse Buttons =============
        // ==========================================================
        let mut bouton_create = Button::new(
            2.0 * b_padding,
            game_button_offsety + b_padding + game_button_sizey,
            game_button_sizex,
            game_button_sizey,
            CREATE.to_string(),
            false,
            scale_factor,
        );
        bouton_create.set_enabled(false);
        button_list.push(bouton_create);
        actions_boutons.insert(
            CREATE.to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.new_game(false, false);
            })),
        );

        let mut bouton_browse = Button::new(
            2.0 * b_padding,
            game_button_offsety + 2. * (b_padding + game_button_sizey),
            game_button_sizex,
            game_button_sizey,
            BROWSE.to_string(),
            false,
            scale_factor,
        );
        bouton_browse.set_clickable(false);
        bouton_browse.set_enabled(false);
        button_list.push(bouton_browse);
        actions_boutons.insert(
            BROWSE.to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.set_mode_btn(false);
                sudoku_display.set_new_game_btn(true);
                sudoku_display.new_game(false, true);
            })),
        );

        // ==========================================================
        // ============== Sovle and Solve Once Buttons ==============
        // ==========================================================
        let button_solve_once = Button::new(
            x_offset + grid_size / 2.0 - button_sizex - b_padding,
            2.0 * b_padding,
            button_sizex,
            button_sizey,
            SOLVE_ONCE.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_solve_once.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::solve_once)),
        );
        button_list.push(button_solve_once);

        let button_solve = Button::new(
            x_offset + grid_size / 2.0 + b_padding,
            2.0 * b_padding,
            button_sizex,
            button_sizey,
            SOLVE.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(SOLVE.to_string(), Rc::new(Box::new(SudokuDisplay::solve)));
        button_list.push(button_solve);

        // ==========================================================
        // ====================== Notes Buttons =====================
        // ==========================================================
        let pad_size = grid_size / 2.;
        let pad_x_offset = x_offset + grid_size + button_sizey;
        let pad_y_offset = y_offset + pad_size / 4.;

        let button_note = Button::new(
            pad_x_offset,
            pad_y_offset,
            pad_size / 3. - b_padding,
            button_sizey,
            NOTE.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_note.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::notes_btn)),
        );
        button_list.push(button_note);

        let button_note_fill = Button::new(
            pad_x_offset + pad_size / 3.,
            pad_y_offset,
            pad_size / 3. - b_padding,
            button_sizey,
            FILL_NOTES.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_note_fill.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.fill_notes_btn(true);
            })),
        );
        button_list.push(button_note_fill);

        let button_undo = Button::new(
            pad_x_offset + 2. * pad_size / 3.,
            pad_y_offset,
            pad_size / 3. - b_padding,
            button_sizey,
            UNDO.to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_undo.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::undo_btn)),
        );
        button_list.push(button_undo);

        // ==========================================================
        // ===================== Number Buttons =====================
        // ==========================================================
        let value_button_size = pad_size / (n as f32);
        for x in 0..carpet.get_n() {
            for y in 0..carpet.get_n() {
                let value1 = y * carpet.get_n() + x + 1;

                let bouton_numero = Button::new(
                    pad_x_offset + x as f32 * value_button_size,
                    pad_y_offset + button_sizey + b_padding + y as f32 * value_button_size,
                    value_button_size - b_padding,
                    value_button_size - b_padding,
                    value1.to_string(),
                    false,
                    scale_factor,
                );

                let button_id = button_list.len();
                actions_boutons.insert(
                    value1.to_string(),
                    Rc::new(Box::new(move |sudoku_display| {
                        sudoku_display.value_btn(button_id, (x, y));
                    })),
                );

                button_list.push(bouton_numero);
            }
        }

        let life_button = Button::new(
            pad_x_offset,
            pad_y_offset + button_sizey + pad_size + b_padding,
            pad_size - b_padding,
            button_sizey,
            format!("Lifes: {lifes}"),
            false,
            scale_factor,
        );
        button_list.push(life_button);

        Self {
            #[cfg(feature = "database")]
            database: None,
            carpet,
            max_height,
            max_width,
            scale_factor,
            grid_size,
            pixel_per_cell,
            selected_cell: None,
            hovered_cell: None,
            x_offset,
            y_offset,
            mode,
            analyse_text,
            player_pboard_history,
            player_pboard,
            note,
            button_list,
            font,
            actions_boutons,
            background_victoire,
            lifes,
            wrong_cell,
            wrong_cell_handle,
            difficulty,
            pattern,
            pattern_list,
            thorus_view,
            correction_board,
            background_defaite,
            last_processed_keys: None,
        }
    }

    // =============================================

    // =============== INIT FUNCTIONS ==============

    // =============================================

    pub fn init(&mut self) {
        self.set_mode(PLAY);
        self.selected_cell = None;
        self.hovered_cell = None;
        self.note = false;
        self.lifes = 300;
        self.player_pboard =
            vec![
                vec![vec![HashSet::new(); self.carpet.get_n2()]; self.carpet.get_n2()];
                self.carpet.get_n_sudokus()
            ];
        self.player_pboard_history.clear();
        self.thorus_view = (0, 0);
    }

    #[cfg(feature = "database")]
    pub fn set_db(&mut self, database: Option<Database>) {
        for button in self.button_list.iter_mut() {
            if button.text.eq(BROWSE) && button.clickable != database.is_some() {
                button.set_clickable(database.is_some());
                self.database = database;
                break;
            }
        }
    }

    // =============================================

    // ============== BUTTON FUNCTIONS =============

    // =============================================

    fn set_new_game_btn(&mut self, status: bool) {
        for button in self.button_list.iter_mut() {
            if button.text == NEW_GAME {
                button.set_enabled(status);
            }
            if button.text == CANCEL {
                button.set_enabled(!status);
            }
        }
    }

    fn set_pattern_btn(&mut self, status: bool) {
        for button in self.button_list.iter_mut() {
            if self
                .pattern_list
                .iter()
                .any(|pattern| pattern.to_string() == button.text)
                || button.text == DECREASE
                || button.text == INCREASE
            {
                button.set_enabled(status);
            }
        }
    }

    fn pattern_size_btn(&mut self, increase: bool) {
        for pattern in self.pattern_list.iter_mut() {
            let old_pattern = *pattern;
            if increase {
                pattern.add_assign(1);
            } else if old_pattern.get_size() > 2 {
                pattern.sub_assign(1);
            }

            // changing button text
            for button in self.button_list.iter_mut() {
                if button.text == old_pattern.to_string() {
                    button.set_text(pattern.to_string());
                }
            }

            // remapping button action
            let new_pattern = *pattern;
            self.actions_boutons.remove(&old_pattern.to_string());
            self.actions_boutons.insert(
                pattern.to_string(),
                Rc::new(Box::new(move |sudoku_display| {
                    sudoku_display.pattern = new_pattern;
                    sudoku_display.set_pattern_btn(false);
                    sudoku_display.set_difficulty_btn(true);
                })),
            );
        }
    }

    fn set_difficulty_btn(&mut self, status: bool) {
        for button in self.button_list.iter_mut() {
            if button.text == EMPTY
                || SudokuDifficulty::iter().any(|diff| button.text.eq(&diff.to_string()))
            {
                button.set_enabled(status);
            }
        }
    }

    fn set_mode_btn(&mut self, status: bool) {
        for button in self.button_list.iter_mut() {
            if button.text == CREATE || button.text == BROWSE {
                button.set_enabled(status);
            }
        }
    }

    fn new_game(&mut self, empty: bool, browse: bool) {
        self.init();

        self.carpet = if empty {
            CarpetSudoku::new(self.carpet.get_n(), self.pattern)
        } else {
            #[cfg(feature = "database")]
            match (browse, &mut self.database) {
                (true, Some(database)) => CarpetSudoku::load_game_from_db(
                    database,
                    self.carpet.get_n(),
                    self.pattern,
                    self.difficulty,
                ),
                _ => CarpetSudoku::generate_new(self.carpet.get_n(), self.pattern, self.difficulty),
            }

            #[cfg(not(feature = "database"))]
            {
                if browse {
                    eprintln!(
						"SudokuDisplay Error: Cannot fetch a game from database because the database feature isn't enabled"
					);
                }
                CarpetSudoku::generate_new(self.carpet.get_n(), self.pattern, self.difficulty)
            }
        };
        if !empty {
            let _ = self.carpet.randomize();
        }

        for button in self.button_list.iter_mut() {
            if button.text == CREATE || button.text == BROWSE {
                button.set_enabled(false);
            } else if button.text == NEW_GAME {
                button.set_clicked(false);
            }
        }

        let mut corrected_board = self.carpet.clone();
        while let Ok((true, _, _)) = corrected_board.rule_solve(None) {}
        self.correction_board = corrected_board
            .get_sudokus()
            .iter()
            .map(|sudoku| sudoku.get_board().clone())
            .collect();
        self.set_new_game_btn(true);
        self.player_pboard_history = Vec::new();
        self.player_pboard =
            vec![
                vec![vec![HashSet::new(); self.carpet.get_n2()]; self.carpet.get_n2()];
                self.carpet.get_n_sudokus()
            ];
        self.analyse_text = vec!["Ready to analyze".to_string()];
    }

    fn set_mode(&mut self, mode: &str) {
        self.mode = mode.to_string();

        for bouton in self.button_list.iter_mut() {
            if bouton.text.eq(ANALYSE) {
                bouton.set_clicked(mode == ANALYSE);
            }
            if bouton.text.eq(PLAY) {
                bouton.set_clicked(mode == PLAY);
            }
        }

        for button in self.button_list.iter_mut() {
            if button.text == NOTE
                || button.text == UNDO
                || button.text == FILL_NOTES
                || button.text.contains("Lifes: ")
            {
                button.set_enabled(mode == PLAY);
            }
        }
    }

    pub fn solve_once(&mut self) {
        let previous_boards = self
            .carpet
            .get_sudokus()
            .iter()
            .map(|sudoku| sudoku.get_board().clone())
            .collect::<Vec<_>>();

        let reponse = self.carpet.rule_solve_until((true, true), None);
        let (_, rules_used) = reponse;
        self.analyse_text.clear();
        for used_rules in rules_used.iter() {
            for (sudoku, rule) in used_rules.iter().enumerate() {
                self.analyse_text.push(format!(
                    "Sudoku {sudoku} used \"{}\"",
                    Sudoku::get_rule_name_by_id(*rule)
                ));
            }
        }
        self.player_pboard_history.clear();

        let board = self
            .carpet
            .get_sudokus()
            .iter()
            .map(|sudoku| sudoku.get_board().clone())
            .collect::<Vec<_>>();
        for sudoku_i in 0..self.carpet.get_n_sudokus() {
            for x in 0..self.carpet.get_n2() {
                for y in 0..self.carpet.get_n2() {
                    if previous_boards[sudoku_i][y][x] != board[sudoku_i][y][x] {
                        self.carpet
                            .get_cell_possibilities_mut(sudoku_i, x, y)
                            .clear();
                        self.player_pboard[sudoku_i][y][x].clear();
                        let value = board[sudoku_i][y][x];
                        for (x1, y1) in self.carpet.get_cell_group(sudoku_i, x, y, All) {
                            if self.carpet.get_cell_value(sudoku_i, x1, y1) == 0 {
                                self.player_pboard[sudoku_i][y1][x1].remove(&value);
                                self.carpet
                                    .get_cell_possibilities_mut(sudoku_i, x1, y1)
                                    .remove(&value);
                            }
                        }
                    }
                }
            }
        }
    }

    fn solve(&mut self) {
        self.carpet.rule_solve_until((false, false), None);
        for sudoku_i in 0..self.carpet.get_n_sudokus() {
            for x in 0..self.carpet.get_n2() {
                for y in 0..self.carpet.get_n2() {
                    self.player_pboard[sudoku_i][y][x].clear();
                }
            }
        }
    }

    fn notes_btn(&mut self) {
        self.note = !self.note;
        for bouton in self.button_list.iter_mut() {
            if bouton.text.eq(NOTE) {
                bouton.set_clicked(!bouton.clicked());
            }
        }
    }

    fn fill_notes_btn(&mut self, easy: bool) {
        let mut changed = false;
        let old_pboard = self.player_pboard.clone();
        for sudoku_i in 0..self.carpet.get_n_sudokus() {
            for x in 0..self.carpet.get_n2() {
                for y in 0..self.carpet.get_n2() {
                    if self.player_pboard[sudoku_i][y][x].is_empty()
                        && self.carpet.get_cell_value(sudoku_i, x, y) == 0
                    {
                        changed = true;
                        if easy {
                            for i in self.carpet.get_cell_possibilities(sudoku_i, x, y) {
                                self.player_pboard[sudoku_i][y][x].insert(i);
                            }
                        } else {
                            for i in 1..=self.carpet.get_n2() {
                                self.player_pboard[sudoku_i][y][x].insert(i);
                            }
                        }
                    }
                }
            }
        }
        if changed {
            self.player_pboard_history.push(old_pboard);
        }
    }

    fn undo_btn(&mut self) {
        if let Some(last_pboard) = self.player_pboard_history.pop() {
            self.player_pboard = last_pboard;
        }
    }

    fn value_btn(&mut self, button_id: usize, (x, y): Coords) {
        if self.selected_cell.is_none() {
            return;
        }
        let (sudoku_i, x1, y1) = self.selected_cell.unwrap();

        let value = y * self.carpet.get_n() + x + 1;
        let current_value = self.carpet.get_cell_value(sudoku_i, x1, y1);

        if current_value != 0 {
            return;
        }

        if self.note {
            self.player_pboard_history.push(self.player_pboard.clone());

            for (sudoku2, x2, y2) in self.carpet.get_twin_cells(sudoku_i, x1, y1) {
                if self.button_list[button_id].clicked {
                    self.player_pboard[sudoku2][y2][x2].remove(&value);
                } else {
                    self.player_pboard[sudoku2][y2][x2].insert(value);
                }
            }
            return;
        }

        if (self.correction_board[sudoku_i][y1][x1] == 0
            && self
                .carpet
                .get_cell_possibilities(sudoku_i, x1, y1)
                .contains(&value))
            || self.correction_board[sudoku_i][y1][x1] == value
        {
            if self.carpet.set_value(sudoku_i, x1, y1, value).is_err() {
                let _ = self.carpet.remove_value(sudoku_i, x1, y1);
            } else {
                self.player_pboard_history.clear();

                for (sudoku2, x2, y2) in self.carpet.get_twin_cells(sudoku_i, x1, y1) {
                    self.player_pboard[sudoku2][y2][x2].clear();
                }
                for (sudoku2, x2, y2) in self.carpet.get_global_cell_group(sudoku_i, x1, y1, All) {
                    for (sudoku3, x3, y3) in self.carpet.get_twin_cells(sudoku2, x2, y2) {
                        if self.carpet.get_cell_value(sudoku3, x3, y3) == 0 {
                            self.player_pboard[sudoku3][y3][x3].remove(&value);
                        }
                    }
                }

                *self.wrong_cell.lock().unwrap() = None;
                return;
            }
        }

        self.lifes -= 1;
        let old_pboard = self.player_pboard.clone();
        if self.player_pboard[sudoku_i][y1][x1].remove(&value) {
            self.player_pboard_history.push(old_pboard);
        }
        *self.wrong_cell.lock().unwrap() = Some((sudoku_i, x1, y1, value));

        let thread_wrong_cell = Arc::clone(&self.wrong_cell);
        let thread_wrong_cell_handle = Arc::clone(&self.wrong_cell_handle);
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(1));
            *thread_wrong_cell.lock().unwrap() = None;
            *thread_wrong_cell_handle.lock().unwrap() = None;
        });
        *self.wrong_cell_handle.lock().unwrap() = Some(handle);
    }

    // =============================================
    // ============== DRAW FUNCTIONS ===============
    // =============================================

    async fn draw_simple_sudoku(&mut self, font: Font, sudoku_i: usize, x1: usize, y1: usize) {
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();
        let sudoku_x_offset = self.x_offset + (x1 as f32) * self.pixel_per_cell;
        let sudoku_y_offset = self.y_offset + (y1 as f32) * self.pixel_per_cell;

        // outline
        draw_rectangle(
            sudoku_x_offset,
            sudoku_y_offset,
            self.pixel_per_cell * (n2 as f32),
            self.pixel_per_cell * (n2 as f32),
            Color::from_hex(0xffffff),
        );

        // draw the hovered cell
        if let Some((hovered_sudoku, hovered_x, hovered_y)) = self.hovered_cell {
            for (hovered_sudoku, hovered_x, hovered_y) in
                self.carpet
                    .get_twin_cells(hovered_sudoku, hovered_x, hovered_y)
            {
                if hovered_sudoku == sudoku_i {
                    draw_rectangle(
                        (hovered_x as f32) * self.pixel_per_cell + sudoku_x_offset,
                        (hovered_y as f32) * self.pixel_per_cell + sudoku_y_offset,
                        self.pixel_per_cell,
                        self.pixel_per_cell,
                        Color::from_hex(0xf1f5f9),
                    );
                }
            }
        }

        // draw the selected cell
        if let Some((selected_sudoku, selected_x, selected_y)) = self.selected_cell {
            let selected_group =
                self.carpet
                    .get_global_cell_group(selected_sudoku, selected_x, selected_y, All);
            for (i, x, y) in selected_group.iter() {
                if *i != sudoku_i {
                    continue;
                }

                draw_rectangle(
                    (*x as f32) * self.pixel_per_cell + sudoku_x_offset,
                    (*y as f32) * self.pixel_per_cell + sudoku_y_offset,
                    self.pixel_per_cell,
                    self.pixel_per_cell,
                    Color::from_hex(0xe4ebf2),
                );
            }

            for (i, x, y) in self
                .carpet
                .get_twin_cells(selected_sudoku, selected_x, selected_y)
            {
                if i != sudoku_i {
                    continue;
                }

                draw_rectangle(
                    (x as f32) * self.pixel_per_cell + sudoku_x_offset,
                    (y as f32) * self.pixel_per_cell + sudoku_y_offset,
                    self.pixel_per_cell,
                    self.pixel_per_cell,
                    Color::from_hex(0xc2ddf8),
                );
            }
        }

        // draw the wrong cell
        if let Some((wrong_sudoku, wrong_x, wrong_y, _)) = *self.wrong_cell.lock().unwrap() {
            if wrong_sudoku == sudoku_i {
                draw_rectangle(
                    (wrong_x as f32) * self.pixel_per_cell + sudoku_x_offset,
                    (wrong_y as f32) * self.pixel_per_cell + sudoku_y_offset,
                    self.pixel_per_cell,
                    self.pixel_per_cell,
                    Color::from_hex(0xed8f98),
                );
            }
        }

        // draw grid
        for i in 0..n2 {
            let i = i as f32;
            // row
            draw_line(
                0.0 + sudoku_x_offset,
                i * self.pixel_per_cell + sudoku_y_offset,
                self.pixel_per_cell * (n2 as f32) + sudoku_x_offset,
                i * self.pixel_per_cell + sudoku_y_offset,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
            // col
            draw_line(
                i * self.pixel_per_cell + sudoku_x_offset,
                0.0 + sudoku_y_offset,
                i * self.pixel_per_cell + sudoku_x_offset,
                self.pixel_per_cell * (n2 as f32) + sudoku_y_offset,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
        }

        for y in 0..n {
            for x in 0..n {
                draw_rectangle_lines(
                    ((x * n) as f32) * self.pixel_per_cell + sudoku_x_offset,
                    ((y * n) as f32) * self.pixel_per_cell + sudoku_y_offset,
                    (n as f32) * self.pixel_per_cell,
                    (n as f32) * self.pixel_per_cell,
                    2.0,
                    Color::from_hex(0x000000),
                );
            }
        }

        // draw numbers
        for (y, line) in self.carpet.get_sudokus()[sudoku_i]
            .get_board()
            .iter()
            .enumerate()
        {
            for (x, &cell) in line.iter().enumerate() {
                if cell == 0 {
                    continue;
                }
                let font_size = ((self.pixel_per_cell as u16) * 2) / 3;
                let text = cell.to_string();
                let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
                let text_x = (x as f32) * self.pixel_per_cell
                    + (self.pixel_per_cell - text_dimensions.width) / 2.0;
                let text_y = (y as f32) * self.pixel_per_cell
                    + (self.pixel_per_cell + text_dimensions.height) / 2.0;
                draw_text_ex(
                    &text,
                    text_x + sudoku_x_offset,
                    text_y + sudoku_y_offset,
                    TextParams {
                        font: Some(&font),
                        font_size,
                        color: Color::from_hex(0x000000),
                        ..Default::default()
                    },
                );
            }
        }

        let pb = if self.mode.eq(PLAY) {
            self.player_pboard[sudoku_i].clone()
        } else {
            self.carpet.get_sudoku_possibility_board(sudoku_i)
        };

        // draw notes
        for x in 0..n2 {
            for (y, pby) in pb.iter().enumerate() {
                if pby[x].is_empty() {
                    continue;
                }
                let font_size = ((self.pixel_per_cell as u16) * 2) / (3 * (n as u16));
                for i in 0..n {
                    for j in 0..n {
                        let number = i * n + j + 1;
                        if !pby[x].contains(&number) {
                            continue;
                        }
                        let text = number.to_string();
                        let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
                        let text_x = (x as f32) * self.pixel_per_cell
                            - self.pixel_per_cell / (n as f32)
                            + (((j as f32) + 1.0) * self.pixel_per_cell) / (n as f32)
                            + (self.pixel_per_cell / (n as f32) - text_dimensions.width) / 2.0;
                        let text_y = (y as f32) * self.pixel_per_cell
                            - self.pixel_per_cell / (n as f32)
                            + (((i as f32) + 1.0) * self.pixel_per_cell) / (n as f32)
                            + (self.pixel_per_cell / (n as f32) + text_dimensions.height) / 2.0;
                        draw_text_ex(
                            &text,
                            text_x + sudoku_x_offset,
                            text_y + sudoku_y_offset,
                            TextParams {
                                font: Some(&font),
                                font_size,
                                color: Color::from_hex(0x000000),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
    }

    async fn draw_samurai_sudoku(&mut self, font: Font) {
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();

        if let Some((sudoku_i, _, _)) = self.selected_cell {
            if sudoku_i >= 1 {
                let mut x1: usize = 1;
                let mut y1: usize = 1;
                for x in 0..2 {
                    for y in 0..2 {
                        let i = 1 + x + y * 2;
                        if i == sudoku_i {
                            x1 = x;
                            y1 = y;
                            continue;
                        }
                        self.draw_simple_sudoku(
                            font.clone(),
                            1 + x + y * 2,
                            (2 * n2 - 2 * n) * x,
                            (2 * n2 - 2 * n) * y,
                        )
                        .await;
                    }
                }
                self.draw_simple_sudoku(font.clone(), 0, n2 - n, n2 - n)
                    .await;
                self.draw_simple_sudoku(
                    font.clone(),
                    sudoku_i,
                    (2 * n2 - 2 * n) * x1,
                    (2 * n2 - 2 * n) * y1,
                )
                .await;
            } else {
                for x in 0..2 {
                    for y in 0..2 {
                        self.draw_simple_sudoku(
                            font.clone(),
                            1 + x + y * 2,
                            (2 * n2 - 2 * n) * x,
                            (2 * n2 - 2 * n) * y,
                        )
                        .await;
                    }
                }
                self.draw_simple_sudoku(font.clone(), 0, n2 - n, n2 - n)
                    .await;
            }
        } else {
            self.draw_simple_sudoku(font.clone(), 0, n2 - n, n2 - n)
                .await;
            for x in 0..2 {
                for y in 0..2 {
                    self.draw_simple_sudoku(
                        font.clone(),
                        1 + x + y * 2,
                        (2 * n2 - 2 * n) * x,
                        (2 * n2 - 2 * n) * y,
                    )
                    .await;
                }
            }
        }
    }

    async fn draw_diag_sudoku(&mut self, dense: bool, font: Font) {
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();
        let n_sudokus = self.carpet.get_n_sudokus();
        if let Some((selected_i, _, _)) = self.selected_cell {
            let (selected_x1, selected_y1) = if dense {
                (selected_i * n, (n_sudokus - selected_i - 1) * n)
            } else {
                (
                    selected_i * (n2 - n),
                    (n_sudokus - selected_i - 1) * (n2 - n),
                )
            };

            for i in 0..n_sudokus {
                if i == selected_i {
                    continue;
                }
                let (x1, y1) = if dense {
                    (i * n, (n_sudokus - i - 1) * n)
                } else {
                    (i * (n2 - n), (n_sudokus - i - 1) * (n2 - n))
                };
                self.draw_simple_sudoku(font.clone(), i, x1, y1).await;
                self.draw_simple_sudoku(font.clone(), selected_i, selected_x1, selected_y1)
                    .await;
            }
        } else {
            for i in 0..n_sudokus {
                let (x1, y1) = if dense {
                    (i * n, (n_sudokus - i - 1) * n)
                } else {
                    (i * (n2 - n), (n_sudokus - i - 1) * (n2 - n))
                };
                self.draw_simple_sudoku(font.clone(), i, x1, y1).await;
            }
        }
    }

    async fn draw_carpet_sudoku(&mut self, dense: bool, font: Font) {
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();
        let n_sudokus = self.carpet.get_n_sudokus();
        let carpet_size = self.carpet.get_pattern().get_size();

        if let Some((selected_i, _, _)) = self.selected_cell {
            let (selected_x1, selected_y1) = if dense {
                (
                    (selected_i % carpet_size) * n,
                    (selected_i / carpet_size) * n,
                )
            } else {
                (
                    (selected_i % carpet_size) * (n2 - n),
                    (selected_i / carpet_size) * (n2 - n),
                )
            };

            for i in 0..n_sudokus {
                if i == selected_i {
                    continue;
                }

                let (x1, y1) = if dense {
                    ((i % carpet_size) * n, (i / carpet_size) * n)
                } else {
                    ((i % carpet_size) * (n2 - n), (i / carpet_size) * (n2 - n))
                };
                self.draw_simple_sudoku(font.clone(), i, x1, y1).await;

                self.draw_simple_sudoku(font.clone(), selected_i, selected_x1, selected_y1)
                    .await;
            }
        } else {
            for i in 0..n_sudokus {
                let (x1, y1) = if dense {
                    ((i % carpet_size) * n, (i / carpet_size) * n)
                } else {
                    ((i % carpet_size) * (n2 - n), (i / carpet_size) * (n2 - n))
                };
                self.draw_simple_sudoku(font.clone(), i, x1, y1).await;
            }
        }
    }

    pub fn get_cell_from_pixel(
        &mut self,
        (pixel_x, pixel_y): (f32, f32),
    ) -> Option<(usize, usize, usize)> {
        let x = ((pixel_x - self.x_offset) / self.pixel_per_cell) as usize;
        let y = ((pixel_y - self.y_offset) / self.pixel_per_cell) as usize;
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();

        if pixel_x < self.x_offset
            || pixel_x > self.x_offset + self.grid_size
            || pixel_y < self.y_offset
            || pixel_y > self.y_offset + self.grid_size
        {
            return None;
        }

        match self.carpet.get_pattern() {
            CarpetPattern::Simple
            | CarpetPattern::Diagonal(1)
            | CarpetPattern::Carpet(1)
            | CarpetPattern::DenseDiagonal(1)
            | CarpetPattern::DenseCarpet(1) => Some((0, x, y)),
            CarpetPattern::Diagonal(size) => {
                let max_n = n2 + (size - 1) * n * (n - 1);
                for i in 0..size {
                    let min_x = i * n * (n - 1);
                    let max_x = min_x + n2;
                    let max_y = max_n - min_x;
                    let min_y = max_y - n2;
                    if x >= min_x && x < max_x && y >= min_y && y < max_y {
                        return Some((i, x - min_x, y - min_y));
                    }
                }
                None
            }
            CarpetPattern::Samurai => {
                if x < n2 {
                    if y < n2 {
                        Some((1, x, y))
                    } else if y >= n2 + n {
                        Some((3, x, y - n2 - n))
                    } else if x >= n * (n - 1) {
                        Some((0, x - n * (n - 1), y - n * (n - 1)))
                    } else {
                        None
                    }
                } else if x < n2 + n {
                    if y >= n * (n - 1) && y < n2 + n * (n - 1) {
                        Some((0, x - n * (n - 1), y - n * (n - 1)))
                    } else {
                        None
                    }
                } else if y < n2 {
                    Some((2, x - n2 - n, y))
                } else if y >= n2 + n {
                    Some((4, x - n2 - n, y - n2 - n))
                } else if x < n2 + n * (n - 1) {
                    Some((0, x - n * (n - 1), y - n * (n - 1)))
                } else {
                    None
                }
            }
            CarpetPattern::Carpet(size) => {
                for y0 in 0..size {
                    for x0 in 0..size {
                        let i = y0 * size + x0;
                        let min_x = x0 * (n2 - n);
                        let max_x = min_x + n2;
                        let min_y = y0 * (n2 - n);
                        let max_y = min_y + n2;
                        if x >= min_x && x < max_x && y >= min_y && y < max_y {
                            return Some((i, x - min_x, y - min_y));
                        }
                    }
                }
                None
            }
            CarpetPattern::DenseDiagonal(size) => {
                let max_n = n2 + (size - 1) * n;
                for i in 0..size {
                    let min_x = i * n;
                    let max_x = min_x + n2;
                    let max_y = max_n - min_x;
                    let min_y = max_y - n2;
                    if x >= min_x && x < max_x && y >= min_y && y < max_y {
                        return Some((i, x - min_x, y - min_y));
                    }
                }
                None
            }
            CarpetPattern::DenseCarpet(size) => {
                for y0 in 0..size {
                    for x0 in 0..size {
                        let i = y0 * size + x0;
                        let min_x = x0 * n;
                        let max_x = min_x + n2;
                        let min_y = y0 * n;
                        let max_y = min_y + n2;
                        if x >= min_x && x < max_x && y >= min_y && y < max_y {
                            return Some((i, x - min_x, y - min_y));
                        }
                    }
                }
                None
            }
            CarpetPattern::Thorus(size) | CarpetPattern::DenseThorus(size) => {
                Some((self.thorus_view.1 * size + self.thorus_view.0, x, y))
            }
            CarpetPattern::Custom(_) => panic!("Custom pattern not implemented"),
        }
    }

    // ==========================================

    // ================= UPDATE =================

    // ==========================================

    pub fn update_scale(&mut self) {
        let n2 = self.carpet.get_n2();
        let n = self.carpet.get_n();

        let ratio = screen_width() / screen_height();
        let ratio_voulu = 411.0 / 245.0;
        if ratio <= ratio_voulu {
            self.scale_factor = screen_width() / self.max_width;
        } else {
            self.scale_factor = screen_height() / self.max_height;
        }

        self.grid_size = 900.0 * self.scale_factor;
        self.pixel_per_cell = match self.carpet.get_pattern() {
            CarpetPattern::Simple | CarpetPattern::Thorus(_) | CarpetPattern::DenseThorus(_) => {
                self.grid_size / n2 as f32
            }
            CarpetPattern::Samurai => self.grid_size / (n2 * 3 - 2 * n) as f32,
            CarpetPattern::Diagonal(size) | CarpetPattern::Carpet(size) => {
                self.grid_size / (n2 + (n2 - n) * (size - 1)) as f32
            }
            CarpetPattern::DenseDiagonal(size) | CarpetPattern::DenseCarpet(size) => {
                self.grid_size / (n2 + n * (size - 1)) as f32
            }
            CarpetPattern::Custom(_) => panic!("Custom pattern not implemented"),
        };

        self.x_offset = 250.0 * self.scale_factor;
        self.y_offset = 150.0 * self.scale_factor;
    }

    pub fn update_selected_buttons(&mut self) {
        if let Some((sudoku_i, x, y)) = self.selected_cell {
            if self.carpet.get_cell_value(sudoku_i, x, y) != 0 {
                for i in 1..=self.carpet.get_n2() {
                    for button in self.button_list.iter_mut() {
                        if button.text == i.to_string() {
                            button.set_clicked(false);
                            button.set_clickable(false);
                        }
                    }
                }
            } else {
                for i in 1..=self.carpet.get_n2() {
                    for button in self.button_list.iter_mut() {
                        if button.text == i.to_string() {
                            button.set_clicked(false);

                            if let Some((wrong_sudoku, wrong_x, wrong_y, wrong_value)) =
                                *self.wrong_cell.lock().unwrap()
                            {
                                if (wrong_sudoku, wrong_x, wrong_y) == (sudoku_i, x, y)
                                    && button.text == wrong_value.to_string()
                                {
                                    button.set_clickable(false);
                                } else {
                                    button.set_clickable(true);
                                }
                            } else {
                                button.set_clickable(true);
                            }
                        }
                    }
                }
                if self.mode.eq(PLAY) {
                    for i in self.player_pboard[sudoku_i][y][x].clone() {
                        for button in self.button_list.iter_mut() {
                            if button.text == i.to_string() {
                                button.set_clicked(true);
                            }
                        }
                    }
                } else {
                    for i in self.carpet.get_cell_possibilities(sudoku_i, x, y).clone() {
                        for button in self.button_list.iter_mut() {
                            if button.text == i.to_string() {
                                button.set_clicked(true);
                            }
                        }
                    }
                }
            }
        } else {
            for i in 1..=self.carpet.get_n2() {
                for button in self.button_list.iter_mut() {
                    if button.text == i.to_string() {
                        button.set_clicked(false);
                        button.set_clickable(true);
                    }
                }
            }
        }
    }

    pub fn move_thorus_view(&mut self, key_pressed: &HashSet<KeyCode>) -> bool {
        let thorus_size = match self.carpet.get_pattern() {
            CarpetPattern::Thorus(size) | CarpetPattern::DenseThorus(size) => size,
            _ => return false,
        };

        if key_pressed.len() != 2 {
            return false;
        }
        let alts = [KeyCode::LeftAlt, KeyCode::RightAlt]
            .into_iter()
            .collect::<HashSet<_>>();
        let alts_pressed = key_pressed.intersection(&alts);
        if alts_pressed.count() != 1 {
            return false;
        }
        let arrows: HashSet<KeyCode> = [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down]
            .into_iter()
            .collect::<HashSet<_>>();
        let mut arrows_pressed = key_pressed.intersection(&arrows);
        if arrows_pressed.clone().count() != 1 {
            return false;
        }

        match *arrows_pressed.next().unwrap() {
            KeyCode::Left => {
                self.thorus_view.0 = if self.thorus_view.0 == 0 {
                    thorus_size - 1
                } else {
                    self.thorus_view.0 - 1
                }
            }
            KeyCode::Right => self.thorus_view.0 = (self.thorus_view.0 + 1) % thorus_size,
            KeyCode::Up => {
                self.thorus_view.1 = if self.thorus_view.1 == 0 {
                    thorus_size - 1
                } else {
                    self.thorus_view.1 - 1
                }
            }
            KeyCode::Down => self.thorus_view.1 = (self.thorus_view.1 + 1) % thorus_size,
            _ => panic!(),
        };

        if let Some((selected_i, selected_x, selected_y)) = self.selected_cell {
            let new_thorus_i = self.thorus_view.1 * thorus_size + self.thorus_view.0;
            let mut new_selected_cell = None;

            for (i, x, y) in self
                .carpet
                .get_twin_cells(selected_i, selected_x, selected_y)
            {
                if i == new_thorus_i {
                    new_selected_cell = Some((i, x, y));
                    break;
                }
            }

            self.selected_cell = new_selected_cell;
        }

        true
    }

    pub fn process_single_key(&mut self, last_key_pressed: KeyCode) -> bool {
        match last_key_pressed {
            KeyCode::Kp1 | KeyCode::Key1 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("1").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp2 | KeyCode::Key2 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("2").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp3 | KeyCode::Key3 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("3").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp4 | KeyCode::Key4 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("4").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp5 | KeyCode::Key5 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("5").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp6 | KeyCode::Key6 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("6").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp7 | KeyCode::Key7 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("7").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp8 | KeyCode::Key8 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("8").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp9 | KeyCode::Key9 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.actions_boutons.get("9").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::N => {
                if let Some(action) = self.actions_boutons.get(NOTE).cloned() {
                    action(self);
                }
            }
            KeyCode::F => {
                if let Some(action) = self.actions_boutons.get(FILL_NOTES).cloned() {
                    action(self);
                }
            }
            KeyCode::U => {
                if let Some(action) = self.actions_boutons.get(UNDO).cloned() {
                    action(self);
                }
            }
            KeyCode::Escape => {
                self.selected_cell = None;
            }
            KeyCode::A => {
                if let Some(action) = self.actions_boutons.get(ANALYSE).cloned() {
                    action(self);
                }
            }
            KeyCode::P => {
                if let Some(action) = self.actions_boutons.get(PLAY).cloned() {
                    action(self);
                }
            }
            KeyCode::S => {
                if let Some(action) = self.actions_boutons.get(SOLVE).cloned() {
                    action(self);
                }
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                let n2 = self.carpet.get_n2();

                if self.selected_cell.is_none() {
                    self.selected_cell = match self.carpet.get_pattern() {
                        CarpetPattern::Thorus(size) | CarpetPattern::DenseThorus(size) => {
                            Some((self.thorus_view.1 * size + self.thorus_view.0, 0, 0))
                        }
                        _ => Some((0, 0, 0)),
                    };
                    return true;
                }

                match self.carpet.get_pattern() {
                    CarpetPattern::Simple
                    | CarpetPattern::DenseThorus(_)
                    | CarpetPattern::Thorus(_) => {
                        let (sudoku_id, mut x1, mut y1) = self.selected_cell.unwrap();
                        match last_key_pressed {
                            KeyCode::Up => {
                                y1 = if y1 == 0 { n2 - 1 } else { y1 - 1 };
                            }
                            KeyCode::Down => {
                                y1 = (y1 + 1) % n2;
                            }
                            KeyCode::Left => {
                                x1 = if x1 == 0 { n2 - 1 } else { x1 - 1 };
                            }
                            KeyCode::Right => {
                                x1 = (x1 + 1) % n2;
                            }
                            _ => (),
                        };
                        self.selected_cell = Some((sudoku_id, x1, y1));
                        return true;
                    }
                    _ => (),
                }

                let (mut sudoku_i, mut x1, mut y1) = self.selected_cell.unwrap();
                let mut modified = false;
                match last_key_pressed {
                    KeyCode::Up => {
                        if y1 > 0 {
                            y1 -= 1;
                            modified = true;
                        }
                    }
                    KeyCode::Down => {
                        if y1 < n2 - 1 {
                            y1 += 1;
                            modified = true;
                        }
                    }
                    KeyCode::Left => {
                        if x1 > 0 {
                            x1 -= 1;
                            modified = true;
                        }
                    }
                    KeyCode::Right => {
                        if x1 < n2 - 1 {
                            x1 += 1;
                            modified = true;
                        }
                    }
                    _ => (),
                };
                if modified {
                    match self.carpet.get_pattern() {
                        CarpetPattern::Thorus(size) | CarpetPattern::DenseThorus(size) => {
                            self.thorus_view = (sudoku_i % size, sudoku_i / size);
                        }
                        _ => (),
                    }
                    self.selected_cell = Some((sudoku_i, x1, y1));
                    return true;
                }

                modified = false;
                for (new_sudoku_i, new_x, new_y) in self.carpet.get_twin_cells(sudoku_i, x1, y1) {
                    match last_key_pressed {
                        KeyCode::Up => {
                            if new_y == 0 {
                                continue;
                            }
                            (sudoku_i, x1, y1) = (new_sudoku_i, new_x, new_y - 1);
                            modified = true;
                        }
                        KeyCode::Down => {
                            if new_y >= n2 - 1 {
                                continue;
                            }
                            (sudoku_i, x1, y1) = (new_sudoku_i, new_x, new_y + 1);
                            modified = true;
                        }
                        KeyCode::Left => {
                            if new_x == 0 {
                                continue;
                            }
                            (sudoku_i, x1, y1) = (new_sudoku_i, new_x - 1, new_y);
                            modified = true;
                        }
                        KeyCode::Right => {
                            if new_x >= n2 - 1 {
                                continue;
                            }
                            (sudoku_i, x1, y1) = (new_sudoku_i, new_x + 1, new_y);
                            modified = true;
                        }
                        _ => (),
                    }

                    if modified {
                        match self.carpet.get_pattern() {
                            CarpetPattern::Thorus(size) | CarpetPattern::DenseThorus(size) => {
                                self.thorus_view = (sudoku_i % size, sudoku_i / size);
                            }
                            _ => (),
                        }
                        self.selected_cell = Some((sudoku_i, x1, y1));
                        return true;
                    }
                }

                let reverse_direction = match last_key_pressed {
                    KeyCode::Up => KeyCode::Down,
                    KeyCode::Down => KeyCode::Up,
                    KeyCode::Left => KeyCode::Right,
                    KeyCode::Right => KeyCode::Left,
                    _ => panic!(),
                };

                modified = true;
                while modified {
                    modified = false;
                    for (new_sudoku_i, new_x, new_y) in self.carpet.get_twin_cells(sudoku_i, x1, y1)
                    {
                        match reverse_direction {
                            KeyCode::Up => {
                                if new_y == 0 {
                                    continue;
                                }
                                (sudoku_i, x1, y1) = (new_sudoku_i, new_x, new_y - 1);
                                modified = true;
                            }
                            KeyCode::Down => {
                                if new_y >= n2 - 1 {
                                    continue;
                                }
                                (sudoku_i, x1, y1) = (new_sudoku_i, new_x, new_y + 1);
                                modified = true;
                            }
                            KeyCode::Left => {
                                if new_x == 0 {
                                    continue;
                                }
                                (sudoku_i, x1, y1) = (new_sudoku_i, new_x - 1, new_y);
                                modified = true;
                            }
                            KeyCode::Right => {
                                if new_x >= n2 - 1 {
                                    continue;
                                }
                                (sudoku_i, x1, y1) = (new_sudoku_i, new_x + 1, new_y);
                                modified = true;
                            }
                            _ => (),
                        }
                    }
                }

                match self.carpet.get_pattern() {
                    CarpetPattern::Thorus(size) | CarpetPattern::DenseThorus(size) => {
                        self.thorus_view = (sudoku_i % size, sudoku_i / size);
                    }
                    _ => (),
                }
                self.selected_cell = Some((sudoku_i, x1, y1));
            }
            _ => return false,
        }
        true
    }

    pub async fn run(&mut self, font: Font) {
        self.update_scale();
        self.update_selected_buttons();

        // BACKGROUND DRAWING
        clear_background(Color::from_hex(0xffffff));
        if self.carpet.is_filled() {
            let bg_width = self.max_width;
            let bg_height =
                self.background_victoire.height() * (bg_width / self.background_victoire.width());
            draw_texture_ex(
                &self.background_victoire,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(bg_width, bg_height)),
                    ..Default::default()
                },
            );
        } else if self.lifes == 0 {
            let bg_width = self.max_width;
            let bg_height =
                self.background_defaite.height() * (bg_width / self.background_defaite.width());
            draw_texture_ex(
                &self.background_defaite,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(bg_width, bg_height)),
                    ..Default::default()
                },
            );
        }

        // MOUSE LOGIC
        let (mouse_x, mouse_y) = (mouse_position().0, mouse_position().1);
        let is_mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        match self.get_cell_from_pixel((mouse_x, mouse_y)) {
            Some(cell) => {
                if is_mouse_pressed {
                    if self.selected_cell != Some(cell) {
                        self.selected_cell = Some(cell);
                    } else {
                        self.selected_cell = None;
                    }
                } else {
                    self.hovered_cell = Some(cell);
                }
            }
            None => self.hovered_cell = None,
        }

        let mut action = None;
        for bouton in self.button_list.iter_mut() {
            if self.mode == ANALYSE && bouton.text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if bouton.text.contains("Lifes: ") {
                bouton.text = format!("Lifes: {}", self.lifes);
            }
            if bouton.text == UNDO {
                if self.player_pboard_history.is_empty() {
                    bouton.set_clickable(false);
                } else {
                    bouton.set_clickable(true);
                }
            }
            bouton.set_scale_factor(self.scale_factor);
            if !bouton.enabled() {
                continue;
            }

            if self.actions_boutons.contains_key(&bouton.text)
                && mouse_x > bouton.x()
                && mouse_x < bouton.x() + bouton.width()
                && mouse_y > bouton.y()
                && mouse_y < bouton.y() + bouton.height()
            {
                if self.wrong_cell_handle.lock().unwrap().is_none()
                    || bouton.text.chars().any(|c| !c.is_ascii_digit())
                {
                    if is_mouse_button_pressed(MouseButton::Left) && bouton.clickable {
                        action = Some(Rc::clone(self.actions_boutons.get(&bouton.text).unwrap()));
                    }
                    bouton.set_hover(true);
                }
            } else {
                bouton.set_hover(false);
            }
            bouton.draw(self.font.clone()).await;
        }

        if let Some(action) = action {
            action(self);
        }

        if self.mode == ANALYSE {
            let font_size = self.pixel_per_cell / 3.;
            let bx_offset = 100. * self.scale_factor - self.pixel_per_cell / 2.;
            for (index, rule) in self.analyse_text.iter().enumerate() {
                draw_text(
                    rule,
                    self.x_offset + self.grid_size + bx_offset,
                    self.y_offset
                        + font_size * (index + 1) as f32
                        + (self.grid_size - font_size * self.analyse_text.len() as f32) / 2.,
                    font_size,
                    Color::from_hex(0x000000),
                );
            }
        }

        // KEYBOARD LOGIC
        let pressed_keys = get_keys_down();
        if pressed_keys.is_empty() {
            self.last_processed_keys = None;
        }

        if self.last_processed_keys.is_none()
            || self.last_processed_keys.unwrap().elapsed() > Duration::from_millis(200)
        {
            self.last_processed_keys = None;
            if self.move_thorus_view(&pressed_keys)
                || pressed_keys.iter().any(|key| self.process_single_key(*key))
            {
                self.last_processed_keys = Some(Instant::now());
            }
        }

        // CARPET DRAWING
        match self.carpet.get_pattern() {
            CarpetPattern::Simple => self.draw_simple_sudoku(font.clone(), 0, 0, 0).await,
            CarpetPattern::Samurai => self.draw_samurai_sudoku(font.clone()).await,
            CarpetPattern::Diagonal(_) => self.draw_diag_sudoku(false, font.clone()).await,
            CarpetPattern::DenseDiagonal(_) => self.draw_diag_sudoku(true, font.clone()).await,
            CarpetPattern::Carpet(_) => self.draw_carpet_sudoku(false, font.clone()).await,
            CarpetPattern::DenseCarpet(_) => self.draw_carpet_sudoku(true, font.clone()).await,
            CarpetPattern::Thorus(_) | CarpetPattern::DenseThorus(_) => {
                let (sudoku_x, sudoku_y) = self.thorus_view;
                let sudoku_i = sudoku_y * self.carpet.get_pattern().get_size() + sudoku_x;
                self.draw_simple_sudoku(font.clone(), sudoku_i, 0, 0).await
            }
            CarpetPattern::Custom(_) => panic!("Custom pattern not implemented"),
        }
    }
}
