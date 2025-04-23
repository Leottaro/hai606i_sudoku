use crate::carpet_sudoku::{CarpetPattern, CarpetSudoku};
#[cfg(feature = "database")]
use crate::database::Database;
use crate::simple_sudoku::{Coords, SudokuDifficulty, SudokuGroups::*};

use super::{Button, ButtonFunction, SudokuDisplay};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

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
        let mode = "Play".to_string();
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
        let difficulty = SudokuDifficulty::Easy;
        let pattern: CarpetPattern = CarpetPattern::Simple;

        // ================== Buttons ==================
        let button_sizex = 150.0 * scale_factor;
        let button_sizey = 100.0 * scale_factor;
        let button_xpadding = 10.0 * scale_factor;
        let choosey_offset = (y_offset - 100.0) / 2.0;
        let b_padding = 10.0;
        let b_size = (pixel_per_cell * 3.0) / 2.0;

        let button_3rd = b_size * n as f32 / 3.0;

        let bouton_play = Button::new(
            x_offset,
            y_offset - choosey_offset - button_sizey,
            button_sizex,
            button_sizey,
            "Play".to_string(),
            true,
            scale_factor,
        );
        actions_boutons.insert(
            bouton_play.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode("Play");
            })),
        );
        button_list.push(bouton_play);

        let button_analyse = Button::new(
            x_offset + button_sizex + button_xpadding,
            y_offset - choosey_offset - button_sizey,
            button_sizex,
            button_sizey,
            "Analyse".to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_analyse.text.clone(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode("Analyse");
            })),
        );
        button_list.push(button_analyse);

        let new_game_btn = Button::new(
            x_offset + (button_sizex + button_xpadding) * 3.0,
            y_offset - choosey_offset - button_sizey,
            button_sizex,
            button_sizey,
            "New Game".to_string(),
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
            x_offset + (button_sizex + button_xpadding) * 3.0,
            y_offset - choosey_offset - button_sizey,
            button_sizex,
            button_sizey,
            "Cancel".to_string(),
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
        // ================== Sudoku Types Buttons ==================
        // ==========================================================
        for (i, sudoku_type) in CarpetPattern::iter_simple().enumerate() {
            let pattern_string = {
                let mut characters = sudoku_type
                    .to_string()
                    .to_lowercase()
                    .chars()
                    .collect::<Vec<_>>();
                characters[0] = characters[0].to_uppercase().nth(0).unwrap();
                characters.into_iter().collect::<String>()
            };
            let offset = 4.0 + (i as f32);

            let mut bouton = Button::new(
                x_offset + (button_sizex + button_xpadding) * offset,
                y_offset - choosey_offset - button_sizey,
                button_sizex,
                button_sizey,
                pattern_string.clone(),
                false,
                scale_factor,
            );
            bouton.set_enabled(false);
            button_list.push(bouton);
            actions_boutons.insert(
                pattern_string,
                Rc::new(Box::new(move |sudoku_display| {
                    sudoku_display.pattern = sudoku_type;
                    sudoku_display.set_pattern_btn(false);
                    sudoku_display.set_difficulty_btn(true);
                })),
            );
        }

        // ==========================================================
        // ================== Difficulty Buttons ====================
        // ==========================================================
        for (i, difficulty) in SudokuDifficulty::iter().enumerate() {
            let diff_string = {
                let mut characters = difficulty
                    .to_string()
                    .to_lowercase()
                    .chars()
                    .collect::<Vec<_>>();
                characters[0] = characters[0].to_uppercase().nth(0).unwrap();
                characters.into_iter().collect::<String>()
            };
            let offset = 4.0 + (i as f32);

            let mut bouton = Button::new(
                x_offset + (button_sizex + button_xpadding) * offset,
                y_offset - choosey_offset - button_sizey,
                button_sizex,
                button_sizey,
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
        // ================== Create and Browse Buttons =============
        // ==========================================================
        let mut bouton_create = Button::new(
            x_offset + (button_sizex + button_xpadding) * 4.0,
            y_offset - choosey_offset - button_sizey,
            button_sizex,
            button_sizey,
            "Create".to_string(),
            false,
            scale_factor,
        );
        bouton_create.set_enabled(false);
        button_list.push(bouton_create);
        actions_boutons.insert(
            "Create".to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.new_game(false);
            })),
        );

        let mut bouton_browse = Button::new(
            x_offset + (button_sizex + button_xpadding) * 5.0,
            y_offset - choosey_offset - button_sizey,
            button_sizex,
            button_sizey,
            "Browse".to_string(),
            false,
            scale_factor,
        );
        bouton_browse.set_clickable(false);
        bouton_browse.set_enabled(false);
        button_list.push(bouton_browse);
        actions_boutons.insert(
            "Browse".to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.set_mode_btn(false);
                sudoku_display.set_new_game_btn(true);
                sudoku_display.new_game(true);
            })),
        );
        // ==========================================================

        let solvex_offset = 50.0 * scale_factor;
        let solve_ypadding = 10.0 * scale_factor;
        let solve1_x = x_offset - solvex_offset - button_sizex;
        let solve1_y = y_offset + (grid_size - button_sizey * 2.0 - solve_ypadding) / 2.0;

        let button_solve_once = Button::new(
            solve1_x,
            solve1_y,
            button_sizex,
            button_sizey,
            "Solve once".to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_solve_once.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::solve_once)),
        );
        button_list.push(button_solve_once);

        let solve2_y = solve1_y + button_sizey + solve_ypadding;
        let button_solve = Button::new(
            solve1_x,
            solve2_y,
            button_sizex,
            button_sizey,
            "Solve".to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert("Solve".to_string(), Rc::new(Box::new(SudokuDisplay::solve)));
        button_list.push(button_solve);

        let bx_offset = 150.0 * scale_factor;
        let button_note = Button::new(
            x_offset + grid_size + bx_offset,
            y_offset + (grid_size - (b_size + b_padding) * (carpet.get_n() as f32)) / 2.0
                - button_sizey
                - solve_ypadding,
            button_3rd,
            button_sizey,
            "Note".to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_note.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::notes_btn)),
        );
        button_list.push(button_note);

        let button_note_fill = Button::new(
            x_offset + grid_size + bx_offset + b_padding + button_3rd,
            y_offset + (grid_size - (b_size + b_padding) * (carpet.get_n() as f32)) / 2.0
                - button_sizey
                - solve_ypadding,
            button_3rd,
            button_sizey,
            "Fill Notes".to_string(),
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
            x_offset + grid_size + bx_offset + (button_3rd + b_padding) * 2.0,
            y_offset + (grid_size - (b_size + b_padding) * (carpet.get_n() as f32)) / 2.0
                - button_sizey
                - solve_ypadding,
            button_3rd,
            button_sizey,
            "Undo".to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_undo.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::undo_btn)),
        );
        button_list.push(button_undo);

        for x in 0..carpet.get_n() {
            for y in 0..carpet.get_n() {
                let value1 = y * carpet.get_n() + x + 1;

                let bouton_numero = Button::new(
                    x_offset + grid_size + bx_offset + (x as f32) * (b_size + b_padding),
                    y_offset
                        + (grid_size - (b_size + b_padding) * (carpet.get_n() as f32)) / 2.0
                        + (y as f32) * (b_size + b_padding),
                    b_size,
                    b_size,
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
            x_offset + grid_size + bx_offset,
            y_offset + grid_size / 2.0 + ((b_size + b_padding) * (carpet.get_n() as f32)) / 2.0,
            (b_size + b_padding) * carpet.get_n() as f32 - b_padding,
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
            player_pboard_history,
            player_pboard,
            note,
            button_list,
            font,
            actions_boutons,
            background_victoire,
            lifes,
            difficulty,
            pattern,
            correction_board,
            background_defaite,
        }
    }

    // =============================================

    // =============== INIT FUNCTIONS ==============

    // =============================================

    pub fn init(&mut self) {
        self.set_mode("Play");
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
    }

    #[cfg(feature = "database")]
    pub fn set_db(&mut self, database: Option<Database>) {
        for button in self.button_list.iter_mut() {
            if button.text.eq("Browse") && button.clickable != database.is_some() {
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
            if button.text == "New Game" {
                button.set_enabled(status);
            }
            if button.text == "Cancel" {
                button.set_enabled(!status);
            }
        }
    }

    fn set_pattern_btn(&mut self, status: bool) {
        for button in self.button_list.iter_mut() {
            if CarpetPattern::iter_simple().any(|pattern| pattern.to_string() == button.text) {
                button.set_enabled(status);
            }
        }
    }

    fn set_difficulty_btn(&mut self, status: bool) {
        for button in self.button_list.iter_mut() {
            if button.text == "Easy"
                || button.text == "Medium"
                || button.text == "Hard"
                || button.text == "Master"
                || button.text == "Extreme"
            {
                button.set_enabled(status);
            }
        }
    }

    fn set_mode_btn(&mut self, status: bool) {
        for button in self.button_list.iter_mut() {
            if button.text == "Create" || button.text == "Browse" {
                button.set_enabled(status);
            }
        }
    }

    fn new_game(&mut self, browse: bool) {
        self.init();

        #[cfg(feature = "database")]
        match (browse, &mut self.database) {
            (true, Some(database)) => {
                self.carpet = CarpetSudoku::load_game_from_db(
                    database,
                    self.carpet.get_n(),
                    self.pattern,
                    self.difficulty,
                );
            }
            _ => {
                self.carpet =
                    CarpetSudoku::generate_new(self.carpet.get_n(), self.pattern, self.difficulty);
            }
        }

        #[cfg(not(feature = "database"))]
        {
            if browse {
                eprintln!(
					"SudokuDisplay Error: Cannot fetch a game from database because the database feature isn't enabled"
				);
            }
            self.carpet =
                CarpetSudoku::generate_new(self.carpet.get_n(), self.pattern, self.difficulty);
        }

        for button in self.button_list.iter_mut() {
            if button.text == "Create" || button.text == "Browse" {
                button.set_enabled(false);
            } else if button.text == "New Game" {
                button.set_clicked(false);
            }
        }

        let mut corrected_board = self.carpet.clone();
        while let Ok((true, _)) = corrected_board.rule_solve(None) {}
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
    }

    fn set_mode(&mut self, mode: &str) {
        self.mode = mode.to_string();

        for bouton in self.button_list.iter_mut() {
            if bouton.text.eq("Analyse") {
                bouton.set_clicked(mode == "Analyse");
            }
            if bouton.text.eq("Play") {
                bouton.set_clicked(mode == "Play");
            }
        }

        for button in self.button_list.iter_mut() {
            if button.text == "Note"
                || button.text == "Undo"
                || button.text == "Fill Notes"
                || button.text.contains("Lifes: ")
            {
                button.set_enabled(mode == "Play");
            }
        }
    }

    pub fn solve_once(&mut self) {
        let previous_boards = self
            .carpet
            .get_sudokus()
            .into_iter()
            .map(|sudoku| sudoku.get_board().clone())
            .collect::<Vec<_>>();
        self.carpet.rule_solve_until((true, true), None);
        let board = self
            .carpet
            .get_sudokus()
            .into_iter()
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
            if bouton.text.eq("Note") {
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
        if self.selected_cell.is_some() {
            let (sudoku_i, x1, y1) = self.selected_cell.unwrap();
            let value = y * self.carpet.get_n() + x + 1;
            if self.note && self.carpet.get_cell_value(sudoku_i, x1, y1) == 0 {
                self.player_pboard_history.push(self.player_pboard.clone());

                for (sudoku2, x2, y2) in self.carpet.get_twin_cells(sudoku_i, x1, y1) {
                    if self.button_list[button_id].clicked {
                        self.player_pboard[sudoku2][y2][x2].remove(&value);
                    } else {
                        self.player_pboard[sudoku2][y2][x2].insert(value);
                    }
                }
            } else if !self.note {
                if self.correction_board[sudoku_i][y1][x1] == value {
                    self.player_pboard_history.clear();
                    if let Err(err) = self.carpet.set_value(sudoku_i, x1, y1, value) {
                        eprintln!("Error setting value: {err}");
                    }

                    for (sudoku2, x2, y2) in self.carpet.get_twin_cells(sudoku_i, x1, y1) {
                        self.player_pboard[sudoku2][y2][x2].clear();
                    }

                    for (sudoku_id, x, y) in
                        self.carpet.get_global_cell_group(sudoku_i, x1, y1, All)
                    {
                        if self.carpet.get_cell_value(sudoku_id, x, y) == 0 {
                            self.player_pboard[sudoku_id][y][x].remove(&value);
                        }
                    }
                } else {
                    self.lifes -= 1;
                    if let Some((i, a, b)) = self.selected_cell {
                        self.draw_cell((i, a, b), Color::from_hex(0xff0000));
                    }
                }
            }
        }
    }

    // =============================================

    // ============== DRAW FUNCTIONS ===============

    // =============================================

    fn draw_cell(&self, (i, x, y): (usize, usize, usize), color: Color) {
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();
        let n_sudokus = self.carpet.get_n_sudokus();
        let x1 = (i * (n2 - n)) as f32;
        let y1 = ((n_sudokus - i - 1) * (n2 - n)) as f32;
        draw_rectangle(
            (x as f32) * self.pixel_per_cell + self.x_offset + x1,
            (y as f32) * self.pixel_per_cell + self.y_offset + y1,
            self.pixel_per_cell,
            self.pixel_per_cell,
            color,
        );
    }

    async fn draw_simple_sudoku(&self, font: Font, sudoku_i: usize, x1: usize, y1: usize) {
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

        if let Some((selected_sudoku, selected_x, selected_y)) = self.selected_cell {
            let selected_group =
                self.carpet
                    .get_global_cell_group(selected_sudoku, selected_x, selected_y, All);
            for (i, x, y) in selected_group.iter() {
                if *i == sudoku_i {
                    draw_rectangle(
                        (*x as f32) * self.pixel_per_cell + sudoku_x_offset,
                        (*y as f32) * self.pixel_per_cell + sudoku_y_offset,
                        self.pixel_per_cell,
                        self.pixel_per_cell,
                        Color::from_hex(0xe4ebf2),
                    );
                }
            }

            if selected_sudoku == sudoku_i {
                draw_rectangle(
                    (selected_x as f32) * self.pixel_per_cell + sudoku_x_offset,
                    (selected_y as f32) * self.pixel_per_cell + sudoku_y_offset,
                    self.pixel_per_cell,
                    self.pixel_per_cell,
                    Color::from_hex(0xc2ddf8),
                );
            }
        }

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

        let pb = if self.mode.eq("Play") {
            self.player_pboard[sudoku_i].clone()
        } else {
            self.carpet.get_sudoku_possibility_board(sudoku_i)
        };

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

    async fn draw_diag_sudoku(&mut self, font: Font) {
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();
        let n_sudokus = self.carpet.get_n_sudokus();
        if let Some((sudoku_i, _, _)) = self.selected_cell {
            let selected_x1 = sudoku_i * (n2 - n);
            let selected_y1 = (n_sudokus - sudoku_i - 1) * (n2 - n);

            for i in 0..n_sudokus {
                if i == sudoku_i {
                    continue;
                }
                let x1 = i * (n2 - n);
                let y1 = (n_sudokus - i - 1) * (n2 - n);
                self.draw_simple_sudoku(font.clone(), i, x1, y1).await;

                self.draw_simple_sudoku(font.clone(), sudoku_i, selected_x1, selected_y1)
                    .await;
            }
        } else {
            for i in 0..n_sudokus {
                let x1 = i * (n2 - n);
                let y1 = (n_sudokus - i - 1) * (n2 - n);
                self.draw_simple_sudoku(font.clone(), i, x1, y1).await;
            }
        }
    }

    async fn draw_carpet_sudoku(&mut self, font: Font) {
        let n = self.carpet.get_n();
        let n2 = self.carpet.get_n2();
        let n_sudokus = self.carpet.get_n_sudokus();

        if let Some((sudoku_i, _, _)) = self.selected_cell {
            let selected_x1 = (sudoku_i % n) * (n2 - n);
            let selected_y1 = (sudoku_i / n) * (n2 - n);

            for i in 0..n_sudokus {
                if i == sudoku_i {
                    continue;
                }

                let x1 = (i % n) * (n2 - n);
                let y1 = (i / n) * (n2 - n);
                self.draw_simple_sudoku(font.clone(), i, x1, y1).await;

                self.draw_simple_sudoku(font.clone(), sudoku_i, selected_x1, selected_y1)
                    .await;
            }
        } else {
            for i in 0..n_sudokus {
                let x1 = (i % n) * (n2 - n);
                let y1 = (i / n) * (n2 - n);
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
            CarpetPattern::Simple | CarpetPattern::Diagonal(1) | CarpetPattern::Carpet(1) => {
                Some((0, x, y))
            }
            CarpetPattern::Double | CarpetPattern::Diagonal(_) => {
                let size = self.carpet.get_n_sudokus();

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
            CarpetPattern::Simple => self.grid_size / n2 as f32,
            CarpetPattern::Samurai => self.grid_size / (n2 * 3 - 2 * n) as f32,
            CarpetPattern::Double | CarpetPattern::Diagonal(_) => {
                let n_sudokus = self.carpet.get_n_sudokus();
                self.grid_size / (((n2 - n) as f32) * n_sudokus as f32 + (n as f32))
            }
            CarpetPattern::Carpet(size) => self.grid_size / (n2 + (size - 1) * (n2 - n)) as f32,
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
                            button.set_clickable(true);
                        }
                    }
                }
                if self.mode.eq("Play") {
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

    pub fn process_keyboard(&mut self, last_key_pressed: KeyCode) {
        match last_key_pressed {
            KeyCode::Kp1 => {
                if let Some(action) = self.actions_boutons.get("1").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp2 => {
                if let Some(action) = self.actions_boutons.get("2").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp3 => {
                if let Some(action) = self.actions_boutons.get("3").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp4 => {
                if let Some(action) = self.actions_boutons.get("4").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp5 => {
                if let Some(action) = self.actions_boutons.get("5").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp6 => {
                if let Some(action) = self.actions_boutons.get("6").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp7 => {
                if let Some(action) = self.actions_boutons.get("7").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp8 => {
                if let Some(action) = self.actions_boutons.get("8").cloned() {
                    action(self);
                }
            }
            KeyCode::Kp9 => {
                if let Some(action) = self.actions_boutons.get("9").cloned() {
                    action(self);
                }
            }
            KeyCode::N => {
                if let Some(action) = self.actions_boutons.get("Note").cloned() {
                    action(self);
                }
            }
            KeyCode::F => {
                if let Some(action) = self.actions_boutons.get("Fill Notes").cloned() {
                    action(self);
                }
            }
            KeyCode::U => {
                if let Some(action) = self.actions_boutons.get("Undo").cloned() {
                    action(self);
                }
            }
            KeyCode::Escape => {
                self.selected_cell = None;
            }
            KeyCode::A => {
                if let Some(action) = self.actions_boutons.get("Analyse").cloned() {
                    action(self);
                }
            }
            KeyCode::P => {
                if let Some(action) = self.actions_boutons.get("Play").cloned() {
                    action(self);
                }
            }
            KeyCode::S => {
                if let Some(action) = self.actions_boutons.get("Solve").cloned() {
                    action(self);
                }
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                if let Some((sudoku_i, x1, y1)) = &mut self.selected_cell {
                    let n2 = self.carpet.get_n2();
                    let twin_cells = self.carpet.get_twin_cells(*sudoku_i, *x1, *y1);
                    if twin_cells.len() == 1 {
                        let mut modified = false;
                        match last_key_pressed {
                            KeyCode::Up => {
                                if *y1 > 0 {
                                    *y1 -= 1;
                                    modified = true;
                                }
                            }
                            KeyCode::Down => {
                                if *y1 < n2 - 1 {
                                    *y1 += 1;
                                    modified = true;
                                }
                            }
                            KeyCode::Left => {
                                if *x1 > 0 {
                                    *x1 -= 1;
                                    modified = true;
                                }
                            }
                            KeyCode::Right => {
                                if *x1 < n2 - 1 {
                                    *x1 += 1;
                                    modified = true;
                                }
                            }
                            _ => (),
                        };
                        if modified {
                            return;
                        }

                        let direction = match last_key_pressed {
                            KeyCode::Up => KeyCode::Down,
                            KeyCode::Down => KeyCode::Up,
                            KeyCode::Left => KeyCode::Right,
                            KeyCode::Right => KeyCode::Left,
                            _ => panic!(),
                        };

                        modified = true;
                        while modified {
                            modified = false;
                            for (new_sudoku_i, new_x, new_y) in
                                self.carpet.get_twin_cells(*sudoku_i, *x1, *y1)
                            {
                                match direction {
                                    KeyCode::Up => {
                                        if new_y == 0 {
                                            continue;
                                        }
                                        (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x, new_y - 1);
                                        modified = true;
                                    }
                                    KeyCode::Down => {
                                        if new_y >= n2 - 1 {
                                            continue;
                                        }
                                        (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x, new_y + 1);
                                        modified = true;
                                    }
                                    KeyCode::Left => {
                                        if new_x == 0 {
                                            continue;
                                        }
                                        (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x - 1, new_y);
                                        modified = true;
                                    }
                                    KeyCode::Right => {
                                        if new_x >= n2 - 1 {
                                            continue;
                                        }
                                        (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x + 1, new_y);
                                        modified = true;
                                    }
                                    _ => (),
                                }
                            }
                        }
                    } else {
                        for (new_sudoku_i, new_x, new_y) in twin_cells {
                            match last_key_pressed {
                                KeyCode::Up => {
                                    if new_y == 0 {
                                        continue;
                                    }
                                    (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x, new_y - 1);
                                }
                                KeyCode::Down => {
                                    if new_y >= n2 - 1 {
                                        continue;
                                    }
                                    (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x, new_y + 1);
                                }
                                KeyCode::Left => {
                                    if new_x == 0 {
                                        continue;
                                    }
                                    (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x - 1, new_y);
                                }
                                KeyCode::Right => {
                                    if new_x >= n2 - 1 {
                                        continue;
                                    }
                                    (*sudoku_i, *x1, *y1) = (new_sudoku_i, new_x + 1, new_y);
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            _ => (),
        }
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
            None => {
                if is_mouse_pressed {
                    self.selected_cell = None;
                }
                self.hovered_cell = None;
            }
        }

        let mut action = None;
        for bouton in self.button_list.iter_mut() {
            if bouton.text.contains("Lifes: ") {
                bouton.text = format!("Lifes: {}", self.lifes);
            }
            if bouton.text == "Undo" {
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
                if is_mouse_button_pressed(MouseButton::Left) && bouton.clickable {
                    action = Some(Rc::clone(self.actions_boutons.get(&bouton.text).unwrap()));
                }
                bouton.set_hover(true);
            } else {
                bouton.set_hover(false);
            }
            bouton.draw(self.font.clone()).await;
        }
        if let Some(action) = action {
            action(self);
        }

        // KEYBOARD LOGIC
        if let Some(last_key_pressed) = get_last_key_pressed() {
            self.process_keyboard(last_key_pressed);
        }

        // CARPET DRAWING
        match self.carpet.get_pattern() {
            CarpetPattern::Simple => {
                self.draw_simple_sudoku(font.clone(), 0, 0, 0).await;
            }
            CarpetPattern::Samurai => {
                self.draw_samurai_sudoku(font.clone()).await;
            }
            CarpetPattern::Double | CarpetPattern::Diagonal(_) => {
                self.draw_diag_sudoku(font.clone()).await;
            }
            CarpetPattern::Carpet(_) => {
                self.draw_carpet_sudoku(font.clone()).await;
            }
            CarpetPattern::Custom(_) => panic!("Custom pattern not implemented"),
        }
    }
}
