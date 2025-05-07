use crate::carpet_sudoku::{CarpetPattern, CarpetSudoku};
#[cfg(feature = "database")]
use crate::database::Database;
use crate::simple_sudoku::{Coords, Sudoku, SudokuDifficulty, SudokuGroups::*};

use super::{Button, ButtonFunction, SudokuDisplay};
use ::rand::rng;
use ::rand::seq::IteratorRandom;
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const PLAY: &str = "Play";
pub const ANALYSE: &str = "Analyse";
pub const SOLVE_ONCE: &str = "Solve\nonce";
pub const SOLVE: &str = "Solve";
pub const HINT: &str = "Hint";

pub const NOTE: &str = "Note";
pub const FILL_NOTES: &str = "Fill\nnotes";
pub const UNDO: &str = "Undo";
pub const REVERT_SOLVE: &str = "Revert\nsolve";

pub const NEW_GAME: &str = "New game";
pub const CANCEL: &str = "Cancel";
pub const EMPTY: &str = "Empty";
pub const INCREASE: &str = "+";
pub const DECREASE: &str = "_";
pub const CREATE: &str = "Create";
pub const BROWSE: &str = "Browse";
pub const COLOR_INDICATOR: &str = "COULEUR";

pub const BACKGROUND_COLOR: u32 = 0xffffff;
pub const HOVERED_COLOR: u32 = 0xf1f5f9;
pub const SELECTED_COLOR: u32 = 0xe4ebf2;
pub const GROUP_COLOR: u32 = 0xc2ddf8;
pub const WRONG_COLOR: u32 = 0xed8f98;
pub const LINE_COLOR: u32 = 0x444444;
pub const FOREGROUND_COLOR: u32 = 0x000000;

pub const COLORS: [u32; 12] = [
    0xffffff, 0xc1c1c1, 0xdb3425, 0xee7930, 0xfbe54d, 0x5cc93b, 0x75fb9c, 0x4faff9, 0x5552ff,
    0x951db3, 0xd070a5, 0x965635,
];

impl SudokuDisplay {
    pub async fn new(n: usize, font: Font) -> Self {
        let max_height = screen_height() * 1.05;
        let max_width = screen_width() * 1.05;
        let scale_factor = 1.0;
        let grid_size = 900. * scale_factor;
        let pixel_per_cell = grid_size / ((n * n) as f32);
        let x_offset = 250. * scale_factor;
        let y_offset = 150. * scale_factor;

        let carpet = CarpetSudoku::new(n, CarpetPattern::Simple);
        let mode = PLAY.to_string();
        let analyse_text = vec!["Ready to analyze".to_string()];
        let hint_text = String::new();
        let history = Vec::new();
        let player_pboard = vec![
            vec![vec![HashMap::new(); carpet.get_n2()]; carpet.get_n2()];
            carpet.get_n_sudokus()
        ];
        let selected_color = COLORS[0];
        let correction_board =
            vec![vec![vec![0; carpet.get_n2()]; carpet.get_n2()]; carpet.get_n_sudokus()];
        let note = true;
        let mut button_list = Vec::new();
        let mut buttons_action: HashMap<String, ButtonFunction> = HashMap::new();
        let lifes = 3;
        let wrong_cell = Arc::new(Mutex::new(None));
        let wrong_cell_handle = Arc::new(Mutex::new(None));
        let difficulty = SudokuDifficulty::Easy;
        let pattern: CarpetPattern = CarpetPattern::Simple;
        let pattern_list = CarpetPattern::iter_simple().collect::<Vec<_>>();
        let torus_view = (0, 0);
        #[cfg(feature = "database")]
        let cloud_texture = load_texture("res/icons/cloud.png").await.unwrap();
        #[cfg(feature = "database")]
        let no_cloud_texture = load_texture("res/icons/no_cloud.png").await.unwrap();

        // ================== Buttons ==================
        let button_sizex = 150. * scale_factor;
        let button_sizey = 100. * scale_factor;
        let b_padding = 10.;

        let button_play = Button {
            x: 2.0 * b_padding,
            y: 2.0 * b_padding,
            width: button_sizex,
            height: button_sizey,
            text: PLAY.to_string(),
            scale_factor,
            clicked: true,
            ..Default::default()
        };
        buttons_action.insert(
            button_play.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode(PLAY);
            })),
        );
        button_list.push(button_play);

        let button_analyse = Button {
            x: 2.0 * b_padding + b_padding + button_sizex,
            y: 2.0 * b_padding,
            width: button_sizex,
            height: button_sizey,
            text: ANALYSE.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(
            button_analyse.text.clone(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode(ANALYSE);
            })),
        );
        button_list.push(button_analyse);

        let game_button_sizex = x_offset - b_padding * 4.;
        let game_button_sizey = button_sizey * 0.9;
        let game_button_offsety = 4.0 * b_padding + button_sizey;

        let new_game_btn = Button {
            x: 2.0 * b_padding,
            y: game_button_offsety,
            width: game_button_sizex,
            height: game_button_sizey,
            text: NEW_GAME.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(
            new_game_btn.text.to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.set_new_game_btn(false);
                sudoku_display.set_pattern_btn(true);
            })),
        );
        button_list.push(new_game_btn);

        let mut new_game_cancel_btn = Button {
            x: 2.0 * b_padding,
            y: game_button_offsety,
            width: game_button_sizex,
            height: game_button_sizey,
            text: CANCEL.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(
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
            let mut bouton = Button {
                x: 2.0 * b_padding,
                y: game_button_offsety + ((i + 1) as f32) * (b_padding + game_button_sizey),
                width: game_button_sizex,
                height: game_button_sizey,
                text: pattern.to_string(),
                scale_factor,
                ..Default::default()
            };
            bouton.set_enabled(false);
            button_list.push(bouton);
            buttons_action.insert(
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
        let mut decrease_button = Button {
            x: 2.0 * b_padding,
            y: game_button_offsety
                + ((pattern_list.len() + 1) as f32) * (b_padding + game_button_sizey),
            width: game_button_sizex / 2.,
            height: game_button_sizey / 3.,
            text: decrease_string.clone(),
            scale_factor,
            ..Default::default()
        };
        decrease_button.set_enabled(false);
        button_list.push(decrease_button);
        buttons_action.insert(
            decrease_string,
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.pattern_size_btn(false);
            })),
        );

        let increase_string = INCREASE.to_string();
        let mut increase_button = Button {
            x: 2.0 * b_padding + game_button_sizex / 2.,
            y: game_button_offsety
                + ((pattern_list.len() + 1) as f32) * (b_padding + game_button_sizey),
            width: game_button_sizex / 2.,
            height: game_button_sizey / 3.,
            text: increase_string.clone(),
            scale_factor,
            ..Default::default()
        };
        increase_button.set_enabled(false);
        button_list.push(increase_button);
        buttons_action.insert(
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
            let mut bouton = Button {
                x: 2.0 * b_padding,
                y: game_button_offsety + ((i + 2) as f32) * (b_padding + game_button_sizey),
                width: game_button_sizex,
                height: game_button_sizey,
                text: diff_string.clone(),
                scale_factor,
                ..Default::default()
            };
            bouton.set_enabled(false);
            button_list.push(bouton);
            buttons_action.insert(
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
        let mut bouton = Button {
            x: 2.0 * b_padding,
            y: game_button_offsety + b_padding + game_button_sizey,
            width: game_button_sizex,
            height: game_button_sizey,
            text: EMPTY.to_string(),
            scale_factor,
            ..Default::default()
        };
        bouton.set_enabled(false);
        button_list.push(bouton);
        buttons_action.insert(
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
        let mut bouton_create = Button {
            x: 2.0 * b_padding,
            y: game_button_offsety + b_padding + game_button_sizey,
            width: game_button_sizex,
            height: game_button_sizey,
            text: CREATE.to_string(),
            scale_factor,
            ..Default::default()
        };
        bouton_create.set_enabled(false);
        button_list.push(bouton_create);
        buttons_action.insert(
            CREATE.to_string(),
            Rc::new(Box::new(move |sudoku_display| {
                sudoku_display.new_game(false, false);
            })),
        );

        let mut bouton_browse = Button {
            x: 2.0 * b_padding,
            y: game_button_offsety + 2. * (b_padding + game_button_sizey),
            width: game_button_sizex,
            height: game_button_sizey,
            text: BROWSE.to_string(),
            scale_factor,
            ..Default::default()
        };
        bouton_browse.set_clickable(false);
        bouton_browse.set_enabled(false);
        button_list.push(bouton_browse);
        buttons_action.insert(
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
        let button_solve_once = Button {
            x: x_offset + grid_size / 2.0 - button_sizex - b_padding,
            y: 2.0 * b_padding,
            width: button_sizex,
            height: button_sizey,
            text: SOLVE_ONCE.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(
            button_solve_once.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::solve_once)),
        );
        button_list.push(button_solve_once);

        let button_solve = Button {
            x: x_offset + grid_size / 2.0,
            y: 2.0 * b_padding,
            width: button_sizex,
            height: button_sizey,
            text: SOLVE.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(SOLVE.to_string(), Rc::new(Box::new(SudokuDisplay::solve)));
        button_list.push(button_solve);

        let button_hint = Button {
            x: x_offset + grid_size / 2.0 + button_sizex + b_padding,
            y: 2.0 * b_padding,
            width: button_sizex,
            height: button_sizey,
            text: HINT.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(HINT.to_string(), Rc::new(Box::new(SudokuDisplay::hint)));
        button_list.push(button_hint);

        let button_revert_solve = Button {
            x: x_offset + grid_size / 2.0 + button_sizex + b_padding,
            y: 2.0 * b_padding,
            width: button_sizex,
            height: button_sizey,
            text: REVERT_SOLVE.to_string(),
            scale_factor,
            enabled: mode == ANALYSE,
            ..Default::default()
        };
        buttons_action.insert(
            REVERT_SOLVE.to_string(),
            Rc::new(Box::new(SudokuDisplay::undo_btn)),
        );
        button_list.push(button_revert_solve);

        // ==========================================================
        // ====================== Notes Buttons =====================
        // ==========================================================
        let pad_size = grid_size / 2.;
        let pad_x_offset = x_offset + grid_size + button_sizey;
        let pad_y_offset = y_offset + pad_size / 10.;

        let button_note = Button {
            x: pad_x_offset,
            y: pad_y_offset,
            width: pad_size / 3. - b_padding,
            height: button_sizey,
            text: NOTE.to_string(),
            scale_factor,
            clicked: note,
            ..Default::default()
        };
        buttons_action.insert(
            button_note.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::notes_btn)),
        );
        button_list.push(button_note);

        let button_note_fill = Button {
            x: pad_x_offset + pad_size / 3.,
            y: pad_y_offset,
            width: pad_size / 3. - b_padding,
            height: button_sizey,
            text: FILL_NOTES.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(
            button_note_fill.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.fill_notes_btn(true);
            })),
        );
        button_list.push(button_note_fill);

        let button_undo = Button {
            x: pad_x_offset + 2. * pad_size / 3.,
            y: pad_y_offset,
            width: pad_size / 3. - b_padding,
            height: button_sizey,
            text: UNDO.to_string(),
            scale_factor,
            ..Default::default()
        };
        buttons_action.insert(
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

                let bouton_numero = Button {
                    x: pad_x_offset + x as f32 * value_button_size,
                    y: pad_y_offset + button_sizey + b_padding + y as f32 * value_button_size,
                    width: value_button_size - b_padding,
                    height: value_button_size - b_padding,
                    text: value1.to_string(),
                    scale_factor,
                    ..Default::default()
                };

                let button_id = button_list.len();
                buttons_action.insert(
                    value1.to_string(),
                    Rc::new(Box::new(move |sudoku_display| {
                        sudoku_display.value_btn(button_id, (x, y));
                    })),
                );

                button_list.push(bouton_numero);
            }
        }

        let life_button = Button {
            x: pad_x_offset,
            y: pad_y_offset + button_sizey + pad_size + b_padding,
            width: pad_size - b_padding,
            height: button_sizey,
            text: format!("Lifes: {lifes}"),
            scale_factor,
            ..Default::default()
        };
        button_list.push(life_button);

        // ==========================================================
        // ===================== Color Buttons ======================
        // ==========================================================

        let color_line = COLORS.len() / 2;
        let color_button_size = (pad_size - color_line as f32 * b_padding) / color_line as f32;
        for (i, color) in COLORS.into_iter().enumerate() {
            let x = (i % color_line) as f32 * (b_padding + color_button_size);
            let y = (i / color_line) as f32 * (b_padding + color_button_size);
            let bouton_couleur = Button {
                x: pad_x_offset + x,
                y: pad_y_offset + 2. * (b_padding + button_sizey) + pad_size + y,
                width: color_button_size,
                height: color_button_size,
                text: color.to_string(),
                scale_factor,
                background_color: Color::from_hex(color),
                draw_text: false,
                draw_border: true,
                stroke: (i == 0),
                ..Default::default()
            };
            buttons_action.insert(
                bouton_couleur.text.clone(),
                Rc::new(Box::new(move |sudoku_display| {
                    sudoku_display.color_btn(color)
                })),
            );
            button_list.push(bouton_couleur);
        }

        // ==========================================================
        // ===================== Color Buttons ======================
        // ==========================================================

        let bouton_indicateur = Button {
            x: pad_x_offset - b_padding - color_button_size / 2.,
            y: pad_y_offset
                + 2. * (b_padding + button_sizey)
                + pad_size
                + b_padding / 2.
                + 3. * color_button_size / 4.,
            width: color_button_size / 2.,
            height: color_button_size / 2.,
            text: COLOR_INDICATOR.to_string(),
            scale_factor,
            background_color: Color::from_hex(COLORS[0]),
            draw_text: false,
            draw_border: true,
            ..Default::default()
        };
        button_list.push(bouton_indicateur);

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
            hint_text,
            history,
            player_pboard,
            selected_color,
            note,
            button_list,
            font,
            buttons_action,
            lifes,
            wrong_cell,
            wrong_cell_handle,
            difficulty,
            pattern,
            pattern_list,
            torus_view,
            correction_board,
            last_processed_keys: None,
            #[cfg(feature = "database")]
            cloud_texture,
            #[cfg(feature = "database")]
            no_cloud_texture,
        }
    }

    // =============================================

    // =============== INIT FUNCTIONS ==============

    // =============================================

    pub fn init(&mut self) {
        self.set_mode(PLAY);
        self.selected_cell = None;
        self.hovered_cell = None;
        if !self.note {
            self.notes_btn();
        }
        self.lifes = 3;
        self.player_pboard =
            vec![
                vec![vec![HashMap::new(); self.carpet.get_n2()]; self.carpet.get_n2()];
                self.carpet.get_n_sudokus()
            ];
        self.history.clear();
        self.hint_text.clear();
        self.analyse_text = vec!["Ready to analyze".to_string()];
        self.torus_view = (0, 0);
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
            self.buttons_action.remove(&old_pattern.to_string());
            self.buttons_action.insert(
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

        let new_carpet;
        if empty {
            new_carpet = Some(CarpetSudoku::new(self.carpet.get_n(), self.pattern));
        } else {
            #[cfg(feature = "database")]
            match (browse, &mut self.database) {
                (true, Some(database)) => {
                    new_carpet = CarpetSudoku::load_game_from_db(
                        database,
                        self.carpet.get_n(),
                        self.pattern,
                        self.difficulty,
                    )
                }
                _ => {
                    new_carpet = Some(CarpetSudoku::generate_new(
                        self.carpet.get_n(),
                        self.pattern,
                        self.difficulty,
                    ))
                }
            }
            #[cfg(not(feature = "database"))]
            {
                if browse {
                    eprintln!(
						"SudokuDisplay Error: Cannot fetch a game from database because the database feature isn't enabled"
					);
                }
                new_carpet = Some(CarpetSudoku::generate_new(
                    self.carpet.get_n(),
                    self.pattern,
                    self.difficulty,
                ))
            }
        }

        if let Some(new_carpet) = new_carpet {
            self.carpet = new_carpet;
            let _ = self.carpet.randomize();
            self.player_pboard =
                vec![
                    vec![vec![HashMap::new(); self.carpet.get_n2()]; self.carpet.get_n2()];
                    self.carpet.get_n_sudokus()
                ];
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
    }

    fn set_mode(&mut self, mode: &str) {
        self.mode = mode.to_string();
        if self.note {
            if let Some(action) = self.buttons_action.get(NOTE).cloned() {
                action(self);
            }
        }

        self.history.clear();
        self.hint_text.clear();

        for button in self.button_list.iter_mut() {
            match button.text.as_str() {
                PLAY => button.set_clicked(mode == PLAY),
                ANALYSE => button.set_clicked(mode == ANALYSE),
                REVERT_SOLVE => button.set_enabled(mode == ANALYSE),
                NOTE | FILL_NOTES | UNDO | HINT => button.set_enabled(mode == PLAY),
                text => {
                    if text.starts_with("Lifes: ") {
                        button.set_enabled(mode == PLAY);
                    }
                }
            }
        }
    }

    pub fn solve_once(&mut self) {
        let (did_anything, rules_used) = self.carpet.rule_solve_until((true, true), None);
        if !did_anything {
            return;
        }

        self.history
            .push((self.carpet.clone(), self.player_pboard.clone()));
        self.analyse_text.clear();

        if self.mode == PLAY {
            self.history.clear();
            self.hint_text.clear();
        }

        for used_rules in rules_used.iter() {
            for (sudoku, rule) in used_rules.iter().enumerate() {
                self.analyse_text.push(format!(
                    "Sudoku {sudoku} used \"{}\"",
                    Sudoku::get_rule_name_by_id(*rule)
                ));
            }
        }

        for sudoku_id in 0..self.carpet.get_n_sudokus() {
            for y in 0..self.carpet.get_n2() {
                for x in 0..self.carpet.get_n2() {
                    for value in 1..=self.carpet.get_n2() {
                        if !self
                            .carpet
                            .get_cell_possibilities(sudoku_id, x, y)
                            .contains(&value)
                        {
                            self.player_pboard[sudoku_id][y][x].remove(&value);
                        }
                    }
                }
            }
        }
    }

    pub fn hint(&mut self) {
        if self.carpet.get_filled_cells() == 0 {
            return;
        }
        if self.mode != PLAY {
            if let Some(action) = self.buttons_action.get(PLAY).cloned() {
                action(self);
            }
        }
        self.hint_text.clear();
        let mut rules_used: HashMap<usize, usize> = HashMap::new();
        for i in 0..self.carpet.get_n_sudokus() {
            if let Some(sudoku) = self.carpet.get_sudoku(i) {
                if let Ok(Some(rule)) = sudoku
                    .clone()
                    .rule_solve(None, Some(self.carpet.get_difficulty()))
                {
                    rules_used.insert(i, rule);
                } else {
                    return;
                }
            }
        }
        match (self.carpet.get_pattern(), self.mode.as_str()) {
            (CarpetPattern::Torus(size), PLAY) | (CarpetPattern::DenseTorus(size), PLAY) => {
                let torus_view = self.torus_view.1 * size + self.torus_view.0;
                if let Some(rule) = rules_used.get(&torus_view) {
                    self.hint_text = format!(
                        "Viewing sudoku can use \"{}\"",
                        Sudoku::get_rule_name_by_id(*rule)
                    );
                } else {
                    self.hint_text = "No hint available".to_string();
                }
            }
            (CarpetPattern::Simple, _) => {
                let rule = rules_used.get(&0).unwrap();
                self.hint_text =
                    format!("Sudoku can use \"{}\"", Sudoku::get_rule_name_by_id(*rule));
            }
            _ => {
                let mut rng = rng();
                let (sudoku, rule) = rules_used.iter().choose(&mut rng).unwrap();
                if self.carpet.get_n_sudokus() > 1 {
                    self.hint_text = format!(
                        "Sudoku {} can use \"{}\"",
                        sudoku,
                        Sudoku::get_rule_name_by_id(*rule)
                    );
                }
            }
        }
    }

    fn solve(&mut self) {
        self.hint_text.clear();
        self.analyse_text.clear();

        let old_state = (self.carpet.clone(), self.player_pboard.clone());
        let (did_something, _) = self.carpet.rule_solve_until((false, false), None);
        if did_something {
            self.history.push(old_state);
        }

        if self.mode == PLAY {
            self.history.clear();
            self.hint_text.clear();
        }

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
            } else if bouton.text.eq(COLOR_INDICATOR) {
                bouton.enabled = self.note;
            } else if let Ok(valeur) = bouton.text.parse::<u32>() {
                if COLORS.contains(&valeur) {
                    bouton.enabled = self.note;
                }
            }
        }
    }

    fn fill_notes_btn(&mut self, easy: bool) {
        let mut changed = false;
        let old_state = (self.carpet.clone(), self.player_pboard.clone());
        for sudoku_i in 0..self.carpet.get_n_sudokus() {
            for x in 0..self.carpet.get_n2() {
                for y in 0..self.carpet.get_n2() {
                    if self.player_pboard[sudoku_i][y][x].is_empty()
                        && self.carpet.get_cell_value(sudoku_i, x, y) == 0
                    {
                        changed = true;
                        if easy {
                            for i in self.carpet.get_cell_possibilities(sudoku_i, x, y) {
                                self.player_pboard[sudoku_i][y][x].insert(i, COLORS[0]);
                            }
                        } else {
                            for i in 1..=self.carpet.get_n2() {
                                self.player_pboard[sudoku_i][y][x].insert(i, COLORS[0]);
                            }
                        }
                    }
                }
            }
        }
        if changed {
            self.history.push(old_state);
        }
    }

    fn undo_btn(&mut self) {
        if let Some((last_carpet, last_pboard)) = self.history.pop() {
            self.carpet = last_carpet;
            self.player_pboard = last_pboard;
        }
        self.hint_text.clear();
        self.analyse_text.clear();
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

        self.history
            .push((self.carpet.clone(), self.player_pboard.clone()));

        if self.note {
            for (sudoku2, x2, y2) in self.carpet.get_twin_cells(sudoku_i, x1, y1) {
                if self.button_list[button_id].clicked {
                    if self.selected_color == self.player_pboard[sudoku2][y2][x2][&value] {
                        self.player_pboard[sudoku2][y2][x2].remove(&value);
                    } else {
                        self.player_pboard[sudoku2][y2][x2].insert(value, self.selected_color);
                    }
                } else {
                    self.player_pboard[sudoku2][y2][x2].insert(value, self.selected_color);
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
                self.history.clear();
                self.hint_text.clear();
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
        let old_state = (self.carpet.clone(), self.player_pboard.clone());
        if self.player_pboard[sudoku_i][y1][x1]
            .remove(&value)
            .is_some()
        {
            self.history.push(old_state);
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

    fn color_btn(&mut self, color: u32) {
        self.selected_color = color;
        for button in self.button_list.iter_mut() {
            if button.text == COLOR_INDICATOR {
                button.background_color = Color::from_hex(color);
            }
        }
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
            Color::from_hex(BACKGROUND_COLOR),
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
                        Color::from_hex(HOVERED_COLOR),
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
                    Color::from_hex(SELECTED_COLOR),
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
                    Color::from_hex(GROUP_COLOR),
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
                    Color::from_hex(WRONG_COLOR),
                );
            }
        }

        // draw grid
        for i in 0..n2 {
            let i = i as f32;
            // row
            draw_line(
                0. + sudoku_x_offset,
                i * self.pixel_per_cell + sudoku_y_offset,
                self.pixel_per_cell * (n2 as f32) + sudoku_x_offset,
                i * self.pixel_per_cell + sudoku_y_offset,
                1.0,
                Color::from_hex(LINE_COLOR),
            );
            // col
            draw_line(
                i * self.pixel_per_cell + sudoku_x_offset,
                0. + sudoku_y_offset,
                i * self.pixel_per_cell + sudoku_x_offset,
                self.pixel_per_cell * (n2 as f32) + sudoku_y_offset,
                1.0,
                Color::from_hex(LINE_COLOR),
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
                    Color::from_hex(FOREGROUND_COLOR),
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
                        color: Color::from_hex(FOREGROUND_COLOR),
                        ..Default::default()
                    },
                );
            }
        }

        let pb = if self.mode.eq(PLAY) {
            self.player_pboard[sudoku_i].clone()
        } else {
            self.carpet
                .get_sudoku_possibility_board(sudoku_i)
                .into_iter()
                .map(|line| {
                    line.into_iter()
                        .map(|values| {
                            values
                                .into_iter()
                                .map(|possibility| (possibility, COLORS[0]))
                                .collect::<HashMap<_, _>>()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };

        // draw notes

        let font_size = ((self.pixel_per_cell as u16) * 2) / (3 * (n as u16));
        for (y, pb_line) in pb.iter().enumerate() {
            for (x, pb_cell) in pb_line.iter().enumerate() {
                if pb_cell.is_empty() {
                    continue;
                }

                for i in 0..n {
                    for j in 0..n {
                        let number = i * n + j + 1;
                        if !pb_cell.contains_key(&number) {
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

                        if pb_cell[&number] != COLORS[0] {
                            draw_rectangle(
                                sudoku_x_offset + text_x - text_dimensions.width * 0.2,
                                sudoku_y_offset + text_y - text_dimensions.height * 1.2,
                                text_dimensions.width * 1.4,
                                text_dimensions.height * 1.4,
                                Color::from_hex(pb_cell[&number]),
                            );
                        }

                        draw_text_ex(
                            &text,
                            sudoku_x_offset + text_x,
                            sudoku_y_offset + text_y,
                            TextParams {
                                font: Some(&font),
                                font_size,
                                color: Color::from_hex(FOREGROUND_COLOR),
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

        match (self.carpet.get_pattern(), self.mode.as_str()) {
            (CarpetPattern::Simple, _)
            | (CarpetPattern::Diagonal(1), _)
            | (CarpetPattern::Carpet(1), _)
            | (CarpetPattern::DenseDiagonal(1), _)
            | (CarpetPattern::DenseCarpet(1), _) => Some((0, x, y)),
            (CarpetPattern::Diagonal(size), _) => {
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
            (CarpetPattern::Samurai, _) => {
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
            (CarpetPattern::Carpet(size), _) | (CarpetPattern::Torus(size), ANALYSE) => {
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
            (CarpetPattern::DenseDiagonal(size), _) => {
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
            (CarpetPattern::DenseCarpet(size), _) | (CarpetPattern::DenseTorus(size), ANALYSE) => {
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
            (CarpetPattern::Torus(size) | CarpetPattern::DenseTorus(size), _) => {
                Some((self.torus_view.1 * size + self.torus_view.0, x, y))
            }
            (CarpetPattern::Custom(_), _) => panic!("Custom pattern not implemented"),
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

        self.grid_size = 900. * self.scale_factor;
        self.pixel_per_cell = match (self.carpet.get_pattern(), self.mode.as_str()) {
            (CarpetPattern::Simple, _)
            | (CarpetPattern::Torus(_) | CarpetPattern::DenseTorus(_), PLAY) => {
                self.grid_size / n2 as f32
            }
            (CarpetPattern::Samurai, _) => self.grid_size / (n2 * 3 - 2 * n) as f32,
            (
                CarpetPattern::Diagonal(size)
                | CarpetPattern::Carpet(size)
                | CarpetPattern::Torus(size),
                _,
            ) => self.grid_size / (n2 + (n2 - n) * (size - 1)) as f32,
            (
                CarpetPattern::DenseDiagonal(size)
                | CarpetPattern::DenseCarpet(size)
                | CarpetPattern::DenseTorus(size),
                _,
            ) => self.grid_size / (n2 + n * (size - 1)) as f32,
            (CarpetPattern::Custom(_), _) => panic!("Custom pattern not implemented"),
        };

        self.x_offset = 250. * self.scale_factor;
        self.y_offset = 150. * self.scale_factor;
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
                    for (i, _color) in self.player_pboard[sudoku_i][y][x].clone() {
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

    pub fn move_torus_view(&mut self, key_pressed: &HashSet<KeyCode>) -> bool {
        let torus_size = match (self.carpet.get_pattern(), self.mode.as_str()) {
            (CarpetPattern::Torus(size) | CarpetPattern::DenseTorus(size), PLAY) => size,
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
                self.torus_view.0 = if self.torus_view.0 == 0 {
                    torus_size - 1
                } else {
                    self.torus_view.0 - 1
                }
            }
            KeyCode::Right => self.torus_view.0 = (self.torus_view.0 + 1) % torus_size,
            KeyCode::Up => {
                self.torus_view.1 = if self.torus_view.1 == 0 {
                    torus_size - 1
                } else {
                    self.torus_view.1 - 1
                }
            }
            KeyCode::Down => self.torus_view.1 = (self.torus_view.1 + 1) % torus_size,
            _ => panic!(),
        };

        if let Some((selected_i, selected_x, selected_y)) = self.selected_cell {
            let new_torus_i = self.torus_view.1 * torus_size + self.torus_view.0;
            let mut new_selected_cell = None;

            for (i, x, y) in self
                .carpet
                .get_twin_cells(selected_i, selected_x, selected_y)
            {
                if i == new_torus_i {
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
                    if let Some(action) = self.buttons_action.get("1").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp2 | KeyCode::Key2 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("2").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp3 | KeyCode::Key3 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("3").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp4 | KeyCode::Key4 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("4").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp5 | KeyCode::Key5 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("5").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp6 | KeyCode::Key6 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("6").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp7 | KeyCode::Key7 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("7").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp8 | KeyCode::Key8 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("8").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::Kp9 | KeyCode::Key9 => {
                if self.wrong_cell_handle.lock().unwrap().is_none() {
                    if let Some(action) = self.buttons_action.get("9").cloned() {
                        action(self);
                    }
                }
            }
            KeyCode::N => {
                if let Some(action) = self.buttons_action.get(NOTE).cloned() {
                    action(self);
                }
            }
            KeyCode::F => {
                if let Some(action) = self.buttons_action.get(FILL_NOTES).cloned() {
                    action(self);
                }
            }
            KeyCode::U => {
                if let Some(action) = self.buttons_action.get(UNDO).cloned() {
                    action(self);
                }
            }
            KeyCode::Escape => {
                self.selected_cell = None;
            }
            KeyCode::A => {
                if let Some(action) = self.buttons_action.get(ANALYSE).cloned() {
                    action(self);
                }
            }
            KeyCode::P => {
                if let Some(action) = self.buttons_action.get(PLAY).cloned() {
                    action(self);
                }
            }
            KeyCode::S => {
                if let Some(action) = self.buttons_action.get(SOLVE).cloned() {
                    action(self);
                }
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                let n2 = self.carpet.get_n2();

                if self.selected_cell.is_none() {
                    self.selected_cell = match (self.carpet.get_pattern(), self.mode.as_str()) {
                        (CarpetPattern::Torus(size) | CarpetPattern::DenseTorus(size), PLAY) => {
                            Some((self.torus_view.1 * size + self.torus_view.0, 0, 0))
                        }
                        _ => Some((0, 0, 0)),
                    };
                    return true;
                }

                match (self.carpet.get_pattern(), self.mode.as_str()) {
                    (CarpetPattern::Simple, _)
                    | (CarpetPattern::DenseTorus(_) | CarpetPattern::Torus(_), PLAY) => {
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
                    if let CarpetPattern::Torus(size) | CarpetPattern::DenseTorus(size) =
                        self.carpet.get_pattern()
                    {
                        self.torus_view = (sudoku_i % size, sudoku_i / size);
                    }
                    self.selected_cell = Some((sudoku_i, x1, y1));
                    return true;
                }

                modified = false;
                for (new_sudoku_i, mut new_x, mut new_y) in
                    self.carpet.get_twin_cells(sudoku_i, x1, y1)
                {
                    match last_key_pressed {
                        KeyCode::Up => {
                            if new_y == 0 {
                                continue;
                            }
                            new_y -= 1;
                            modified = true;
                        }
                        KeyCode::Down => {
                            if new_y >= n2 - 1 {
                                continue;
                            }
                            new_y += 1;
                            modified = true;
                        }
                        KeyCode::Left => {
                            if new_x == 0 {
                                continue;
                            }
                            new_x -= 1;
                            modified = true;
                        }
                        KeyCode::Right => {
                            if new_x >= n2 - 1 {
                                continue;
                            }
                            new_x += 1;
                            modified = true;
                        }
                        _ => (),
                    }

                    if modified {
                        if let CarpetPattern::Torus(size) | CarpetPattern::DenseTorus(size) =
                            self.carpet.get_pattern()
                        {
                            self.torus_view = (new_sudoku_i % size, new_sudoku_i / size);
                        }
                        self.selected_cell = Some((new_sudoku_i, new_x, new_y));
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

                if let CarpetPattern::Torus(size) | CarpetPattern::DenseTorus(size) =
                    self.carpet.get_pattern()
                {
                    self.torus_view = (sudoku_i % size, sudoku_i / size);
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
        clear_background(Color::from_hex(BACKGROUND_COLOR));

        #[cfg(feature = "database")]
        if self.database.is_some() {
            draw_texture(
                &self.cloud_texture,
                50.,
                screen_height() - 50.,
                Color::from_hex(0x00ff00),
            );
        } else {
            draw_texture(
                &self.no_cloud_texture,
                50.,
                screen_height() - 50.,
                Color::from_hex(0xff0000),
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
                        if let CarpetPattern::Torus(size) | CarpetPattern::DenseTorus(size) =
                            self.carpet.get_pattern()
                        {
                            self.torus_view = (cell.0 % size, cell.0 / size);
                        }
                    } else {
                        self.selected_cell = None;
                    }
                } else {
                    self.hovered_cell = Some(cell);
                }
            }
            None => self.hovered_cell = None,
        }

        // BUTTONS DRAWING
        let mut action = None;
        for bouton in self.button_list.iter_mut() {
            if self.mode == ANALYSE && bouton.text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if bouton.text.contains("Lifes: ") {
                bouton.text = format!("Lifes: {}", self.lifes);
            }
            if bouton.text == UNDO || bouton.text == REVERT_SOLVE {
                if self.history.is_empty() {
                    bouton.set_clickable(false);
                } else {
                    bouton.set_clickable(true);
                }
            }
            bouton.set_scale_factor(self.scale_factor);
            if !bouton.enabled() {
                continue;
            }

            if self.buttons_action.contains_key(&bouton.text)
                && mouse_x > bouton.x()
                && mouse_x < bouton.x() + bouton.width()
                && mouse_y > bouton.y()
                && mouse_y < bouton.y() + bouton.height()
            {
                if self.wrong_cell_handle.lock().unwrap().is_none()
                    || bouton.text.chars().any(|c| !c.is_ascii_digit())
                {
                    if is_mouse_button_pressed(MouseButton::Left) && bouton.clickable {
                        action = Some(Rc::clone(self.buttons_action.get(&bouton.text).unwrap()));
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
            let font_size = self.grid_size / 45.;
            let bx_offset = 100. * self.scale_factor - self.pixel_per_cell / 2.;
            for (index, rule) in self.analyse_text.iter().enumerate() {
                draw_text_ex(
                    rule,
                    self.x_offset + self.grid_size + bx_offset,
                    self.y_offset
                        + font_size * (index + 1) as f32
                        + (self.grid_size - font_size * self.analyse_text.len() as f32) / 2.,
                    TextParams {
                        font: Some(&font),
                        font_size: font_size as u16,
                        color: Color::from_hex(FOREGROUND_COLOR),
                        ..Default::default()
                    },
                );
            }
        } else {
            let font_size = self.grid_size / 40.;
            draw_text_ex(
                &self.hint_text,
                self.x_offset + self.grid_size / 2.0 + (320. * self.scale_factor) + 10.,
                80. * self.scale_factor,
                TextParams {
                    font: Some(&font),
                    font_size: font_size as u16,
                    color: Color::from_hex(FOREGROUND_COLOR),
                    ..Default::default()
                },
            );
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
            if self.move_torus_view(&pressed_keys)
                || pressed_keys.iter().any(|key| self.process_single_key(*key))
            {
                self.last_processed_keys = Some(Instant::now());
            }
        }

        // CARPET DRAWING
        match (self.carpet.get_pattern(), self.mode.as_str()) {
            (CarpetPattern::Simple, _) => self.draw_simple_sudoku(font.clone(), 0, 0, 0).await,
            (CarpetPattern::Samurai, _) => self.draw_samurai_sudoku(font.clone()).await,
            (CarpetPattern::Diagonal(_), _) => self.draw_diag_sudoku(false, font.clone()).await,
            (CarpetPattern::DenseDiagonal(_), _) => self.draw_diag_sudoku(true, font.clone()).await,
            (CarpetPattern::Carpet(_), _) => self.draw_carpet_sudoku(false, font.clone()).await,
            (CarpetPattern::DenseCarpet(_), _) => self.draw_carpet_sudoku(true, font.clone()).await,
            (CarpetPattern::Torus(_) | CarpetPattern::DenseTorus(_), PLAY) => {
                let (sudoku_x, sudoku_y) = self.torus_view;
                let sudoku_i = sudoku_y * self.carpet.get_pattern().get_size() + sudoku_x;
                self.draw_simple_sudoku(font.clone(), sudoku_i, 0, 0).await
            }
            (CarpetPattern::Torus(_), _) => self.draw_carpet_sudoku(false, font.clone()).await,
            (CarpetPattern::DenseTorus(_), _) => self.draw_carpet_sudoku(true, font.clone()).await,
            (CarpetPattern::Custom(_), _) => panic!("Custom pattern not implemented"),
        }
    }
}
