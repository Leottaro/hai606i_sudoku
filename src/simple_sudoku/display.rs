#[cfg(feature = "database")]
use crate::database::Database;

use super::{
    Button, ButtonFunction, Coords, Sudoku, SudokuDifficulty, SudokuDisplay, SudokuGroups::*,
};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

impl SudokuDisplay {
    pub async fn new(sudoku: Sudoku, font: Font) -> Self {
        let max_height = screen_height() * 1.05;
        let max_width = screen_width() * 1.05;
        let scale_factor = 1.0;
        let grid_size = 900.0 * scale_factor;
        let pixel_per_cell = grid_size / sudoku.get_n2() as f32;
        let x_offset = 250.0 * scale_factor;
        let y_offset = 150.0 * scale_factor;

        let mode = "Play".to_string();
        let player_pboard_history = Vec::new();
        let player_pboard = vec![vec![HashSet::new(); sudoku.get_n2()]; sudoku.get_n2()];
        let correction_board = vec![vec![1; sudoku.get_n2()]; sudoku.get_n2()];
        let note = false;
        let mut button_list = Vec::new();
        let mut actions_boutons: HashMap<String, ButtonFunction> = HashMap::new();
        let background_victoire = load_texture("./res/bg/bg-petit.png").await.unwrap();
        let background_defaite = load_texture("./res/bg/bg-def.png").await.unwrap();
        let lifes = 3;
        let new_game_available = false;
        let difficulty = SudokuDifficulty::Easy;

        // ================== Buttons ==================
        let button_sizex = 150.0 * scale_factor;
        let button_sizey = 100.0 * scale_factor;
        let button_xpadding = 10.0 * scale_factor;
        let choosey_offset = (y_offset - 100.0) / 2.0;

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
            new_game_available,
            scale_factor,
        );
        actions_boutons.insert(
            new_game_btn.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::new_game_btn)),
        );
        button_list.push(new_game_btn);

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
            let offset = 4.0 + i as f32;

            let mut bouton = Button::new(
                x_offset + (button_sizex + button_xpadding) * offset,
                y_offset - choosey_offset - button_sizey,
                button_sizex,
                button_sizey,
                diff_string.clone(),
                false,
                scale_factor,
            );
            bouton.set_enabled(new_game_available);
            button_list.push(bouton);
            actions_boutons.insert(
                diff_string,
                Rc::new(Box::new(move |sudoku_display| {
                    sudoku_display.difficulty_btn(difficulty);
                })),
            );
        }

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
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.new_game(sudoku_display.difficulty, false);
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
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.new_game(sudoku_display.difficulty, true);
            })),
        );

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
        let b_size = pixel_per_cell * 3.0 / 2.0;
        let b_padding = 10.0;
        let button_note = Button::new(
            x_offset + grid_size + bx_offset,
            y_offset + (grid_size - (b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0
                - button_sizey
                - solve_ypadding,
            button_sizex,
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
            x_offset + grid_size + bx_offset + b_padding + button_sizex,
            y_offset + (grid_size - (b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0
                - button_sizey
                - solve_ypadding,
            button_sizex,
            button_sizey,
            "Fill Notes".to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            button_note_fill.text.to_string(),
            Rc::new(Box::new(SudokuDisplay::fill_notes_btn)),
        );
        button_list.push(button_note_fill);

        let button_undo = Button::new(
            x_offset + grid_size + bx_offset + (b_padding + button_sizex) * 2.0,
            y_offset + (grid_size - (b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0
                - button_sizey
                - solve_ypadding,
            b_size,
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

        for x in 0..sudoku.get_n() {
            for y in 0..sudoku.get_n() {
                let value1 = y * sudoku.get_n() + x + 1;

                let bouton_numero = Button::new(
                    x_offset + grid_size + bx_offset + (x as f32) * (b_size + b_padding),
                    y_offset
                        + (grid_size - (b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0
                        + (y as f32) * (b_size + b_padding),
                    b_size,
                    b_size,
                    value1.to_string(),
                    false,
                    scale_factor,
                );

                actions_boutons.insert(
                    value1.to_string(),
                    Rc::new(Box::new(move |sudoku_display| {
                        sudoku_display.value_btn((x, y));
                    })),
                );

                button_list.push(bouton_numero);
            }
        }

        let life_button = Button::new(
            x_offset + grid_size + bx_offset,
            y_offset + (grid_size / 2.) + ((b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0,
            b_size * 3. + b_padding * 2.,
            button_sizey,
            format!("Lifes: {lifes}"),
            false,
            scale_factor,
        );
        button_list.push(life_button);

        Self {
            #[cfg(feature = "database")]
            database: None,
            sudoku,
            max_height,
            max_width,
            scale_factor,
            grid_size,
            pixel_per_cell,
            selected_cell: None,
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
            new_game_available,
            difficulty,
            correction_board,
            background_defaite,
        }
    }

    // =============================================

    // =============== INIT FUNCTIONS ==============

    // =============================================

    pub fn init(&mut self) {
        self.set_mode("Play");
        self.lifes = 3;
        self.selected_cell = None;
        self.player_pboard_history.clear();
        self.player_pboard = vec![vec![HashSet::new(); self.sudoku.get_n2()]; self.sudoku.get_n2()];
        self.note = false;
        self.new_game_available = false;
        self.difficulty = SudokuDifficulty::Easy;
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

    fn new_game_btn(&mut self) {
        let nom_boutons: Vec<String> = SudokuDifficulty::iter()
            .map(|diff| diff.to_string())
            .collect();
        self.new_game_available = !self.new_game_available;
        for bouton in self.button_list.iter_mut() {
            if bouton.text.eq("Create") || bouton.text.eq("Browse") {
                bouton.set_enabled(false);
            }

            if nom_boutons.contains(&bouton.text.to_uppercase()) {
                bouton.set_enabled(self.new_game_available);
            } else if bouton.text == "New Game" {
                bouton.set_clicked(self.new_game_available);
            }
        }
    }

    fn difficulty_btn(&mut self, difficulty: SudokuDifficulty) {
        self.difficulty = difficulty;
        for button in self.button_list.iter_mut() {
            if button.text == "Create" || button.text == "Browse" {
                button.set_enabled(true);
            }
            if button.text == "Easy"
                || button.text == "Medium"
                || button.text == "Hard"
                || button.text == "Master"
                || button.text == "Extreme"
            {
                button.set_enabled(false);
            }
        }
    }

    fn new_game(&mut self, difficulty: SudokuDifficulty, browse: bool) {
        self.init();
        #[cfg(feature = "database")]
        match (browse, &mut self.database) {
            (true, Some(database)) => {
                self.sudoku = Sudoku::load_game_from_db(database, self.sudoku.get_n(), difficulty)
            }
            _ => self.sudoku = Sudoku::generate_new(self.sudoku.get_n(), difficulty).unwrap(),
        };

        #[cfg(not(feature = "database"))]
        if browse {
            eprintln!("SudokuDisplay Error: Cannot fetch a game from database because the database feature isn't enabled");
            self.sudoku = Sudoku::generate_new(self.sudoku.n, difficulty).unwrap()
        } else {
            self.sudoku = Sudoku::generate_new(self.sudoku.n, difficulty).unwrap()
        };

        for button in self.button_list.iter_mut() {
            if button.text == "Create" || button.text == "Browse" {
                button.set_enabled(false);
            } else if button.text == "New Game" {
                button.set_clicked(false);
            }
        }

        self.correction_board = self.sudoku.solve();
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
        let previous_board = self.sudoku.get_board().clone();
        loop {
            match self.sudoku.rule_solve(None, None) {
                Ok(None | Some(0) | Some(1)) => break,
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{err}");
                }
            }
        }
        let board = self.sudoku.get_board().clone();
        for x in 0..self.sudoku.get_n2() {
            for y in 0..self.sudoku.get_n2() {
                if previous_board[y][x] != board[y][x] {
                    self.sudoku.get_cell_possibilities_mut(x, y).clear();
                    self.player_pboard[y][x].clear();
                    let value = board[y][x];
                    for (x1, y1) in self.sudoku.get_cell_group(x, y, All) {
                        if self.sudoku.get_cell_value(x1, y1) == 0 {
                            self.player_pboard[y1][x1].remove(&value);
                            self.sudoku
                                .get_cell_possibilities_mut(x1, y1)
                                .remove(&value);
                        }
                    }
                }
            }
        }
    }

    fn solve(&mut self) {
        while !self.sudoku.is_filled() {
            self.solve_once();
        }
    }

    fn notes_btn(&mut self) {
        self.note = !self.note;
        for bouton in self.button_list.iter_mut() {
            if bouton.text == *"Note" {
                bouton.set_clicked(!bouton.clicked());
            }
        }
    }

    fn fill_notes_btn(&mut self) {
        let mut changed = false;
        let old_pboard = self.player_pboard.clone();
        for x in 0..self.sudoku.get_n2() {
            for y in 0..self.sudoku.get_n2() {
                if self.player_pboard[y][x].is_empty() && self.sudoku.get_cell_value(x, y) == 0 {
                    changed = true;
                    for i in 1..=self.sudoku.get_n2() {
                        self.player_pboard[y][x].insert(i);
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

    fn value_btn(&mut self, (x, y): Coords) {
        if self.selected_cell.is_some() {
            let (x1, y1) = self.selected_cell.unwrap();
            let value = y * self.sudoku.get_n() + x + 1;
            if self.note && self.sudoku.get_cell_value(x1, y1) == 0 {
                self.player_pboard_history.push(self.player_pboard.clone());
                for bouton in self.button_list.iter_mut() {
                    if bouton.text == value.to_string() {
                        if bouton.clicked {
                            self.player_pboard[y1][x1].remove(&value);
                        } else {
                            self.player_pboard[y1][x1].insert(value);
                        }
                    }
                }
            } else if !self.note {
                if self.correction_board[y1][x1] == value {
                    self.player_pboard_history.clear();
                    info!("Bonne réponse !");
                    self.sudoku.set_value(x1, y1, value).unwrap();
                    self.player_pboard[y1][x1].clear();

                    for (x, y) in self.sudoku.get_cell_group(x1, y1, All) {
                        if self.sudoku.get_cell_value(x, y) == 0 {
                            self.player_pboard[y][x].remove(&value);
                            self.sudoku.get_cell_possibilities_mut(x, y).remove(&value);
                        }
                    }
                } else {
                    self.lifes -= 1;
                    if let Some(coords) = self.selected_cell {
                        self.draw_cell(coords, Color::from_hex(0xff0000));
                    }
                }
            }
        }
    }

    // =============================================

    // ============== DRAW FUNCTIONS ===============

    // =============================================

    fn draw_cell(&self, (x, y): Coords, color: Color) {
        draw_rectangle(
            x as f32 * self.pixel_per_cell + self.x_offset,
            y as f32 * self.pixel_per_cell + self.y_offset,
            self.pixel_per_cell,
            self.pixel_per_cell,
            color,
        );
    }

    async fn draw_simple_sudoku(&self, font: Font, x1: usize, y1: usize) {
        let n = self.sudoku.get_n();
        let n2 = self.sudoku.get_n2();
        let sudoku_x_offset = self.x_offset + x1 as f32 * self.pixel_per_cell;
        let sudoku_y_offset = self.y_offset + y1 as f32 * self.pixel_per_cell;
        for i in 0..n2 {
            let i = i as f32;
            // row
            draw_line(
                0.0 + sudoku_x_offset,
                i * self.pixel_per_cell + sudoku_y_offset,
                self.pixel_per_cell * n2 as f32 + sudoku_x_offset,
                i * self.pixel_per_cell + sudoku_y_offset,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
            // col
            draw_line(
                i * self.pixel_per_cell + sudoku_x_offset,
                0.0 + sudoku_y_offset,
                i * self.pixel_per_cell + sudoku_x_offset,
                self.pixel_per_cell * n2 as f32 + sudoku_y_offset,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
        }

        for y in 0..n {
            for x in 0..n {
                draw_rectangle_lines(
                    (x * n) as f32 * self.pixel_per_cell + sudoku_x_offset,
                    (y * n) as f32 * self.pixel_per_cell + sudoku_y_offset,
                    n as f32 * self.pixel_per_cell,
                    n as f32 * self.pixel_per_cell,
                    2.0,
                    Color::from_hex(0x000000),
                );
            }
        }

        for (y, line) in self.sudoku.get_board().iter().enumerate() {
            for (x, &cell) in line.iter().enumerate() {
                if cell == 0 {
                    continue;
                }
                let font_size = self.pixel_per_cell as u16 * 2 / 3;
                let text = cell.to_string();
                let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
                let text_x = (x as f32 * self.pixel_per_cell)
                    + (self.pixel_per_cell - text_dimensions.width) / 2.0;
                let text_y = (y as f32 * self.pixel_per_cell)
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

        let pb = if self.mode == *"Play" {
            self.player_pboard.clone()
        } else {
            self.sudoku.get_possibility_board().clone()
        };

        for x in 0..n2 {
            for (y, pby) in pb.iter().enumerate() {
                if pby[x].is_empty() {
                    continue;
                }
                let font_size = self.pixel_per_cell as u16 * 2 / (3 * n as u16);
                for i in 0..n {
                    for j in 0..n {
                        let number = i * n + j + 1;
                        if !pby[x].contains(&number) {
                            continue;
                        }
                        let text = number.to_string();
                        let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
                        let text_x = (x as f32 * self.pixel_per_cell)
                            - (self.pixel_per_cell / n as f32)
                            + ((j as f32 + 1.0) * self.pixel_per_cell / n as f32)
                            + (self.pixel_per_cell / n as f32 - text_dimensions.width) / 2.0;
                        let text_y = (y as f32 * self.pixel_per_cell)
                            - (self.pixel_per_cell / n as f32)
                            + ((i as f32 + 1.0) * self.pixel_per_cell / n as f32)
                            + (self.pixel_per_cell / n as f32 + text_dimensions.height) / 2.0;
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

    async fn draw_diag_sudoku(&mut self, font: Font, taille: usize) {
        let n = self.sudoku.get_n();
        let n2 = self.sudoku.get_n2();
        for i in 0..taille {
            let x1 = i * (n2 - n);
            let y1 = (taille - i - 1) * (n2 - n);
            self.draw_simple_sudoku(font.clone(), x1, y1).await;
            self.draw_simple_sudoku(font.clone(), x1, y1).await;
        }
    }

    // ==========================================

    // ================= UPDATE =================

    // ==========================================

    pub fn update_scale(&mut self) {
        let ratio = screen_width() / screen_height();
        let ratio_voulu = 411. / 245.;
        if ratio <= ratio_voulu {
            self.scale_factor = screen_width() / self.max_width;
        } else {
            self.scale_factor = screen_height() / self.max_height;
        }

        self.grid_size = 900.0 * self.scale_factor;
        //self.pixel_per_cell = self.grid_size / self.sudoku.get_n2() as f32;
        self.pixel_per_cell = self.grid_size
            / (((self.sudoku.get_n2() - self.sudoku.get_n()) as f32) * 3.0
                + self.sudoku.get_n() as f32);
        self.x_offset = 250.0 * self.scale_factor;
        self.y_offset = 150.0 * self.scale_factor;
    }

    pub fn update_selected_buttons(&mut self) {
        if let Some((x, y)) = self.selected_cell {
            if self.sudoku.get_cell_value(x, y) != 0 {
                for i in 1..=self.sudoku.get_n2() {
                    for button in self.button_list.iter_mut() {
                        if button.text == i.to_string() {
                            button.set_clicked(false);
                            button.set_clickable(false);
                        }
                    }
                }
            } else {
                for i in 1..=self.sudoku.get_n2() {
                    for button in self.button_list.iter_mut() {
                        if button.text == i.to_string() {
                            button.set_clicked(false);
                            button.set_clickable(true);
                        }
                    }
                }
                if self.mode == "Play".to_string() {
                    for i in self.player_pboard[y][x].clone() {
                        for button in self.button_list.iter_mut() {
                            if button.text == i.to_string() {
                                button.set_clicked(true);
                            }
                        }
                    }
                } else {
                    for i in self.sudoku.possibility_board[y][x].clone() {
                        for button in self.button_list.iter_mut() {
                            if button.text == i.to_string() {
                                button.set_clicked(true);
                            }
                        }
                    }
                }
            }
        } else {
            for i in 1..=self.sudoku.get_n2() {
                for button in self.button_list.iter_mut() {
                    if button.text == i.to_string() {
                        button.set_clicked(false);
                        button.set_clickable(true);
                    }
                }
            }
        }
    }

    pub fn diag_click(&mut self, taille: usize, x: usize, y: usize) {
        if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y) {
            self.selected_cell = None;
        }
        else{
            self.selected_cell = Some((0,0));
        }
    }

    pub async fn run(&mut self, font: Font) {
        self.update_scale();
        self.update_selected_buttons();

        let (mouse_x, mouse_y) = (mouse_position().0, mouse_position().1);
        let x = ((mouse_x - self.x_offset) / self.pixel_per_cell) as usize;
        let y = ((mouse_y - self.y_offset) / self.pixel_per_cell) as usize;

        //test bg

        clear_background(Color::from_hex(0xffffff));

        if self.sudoku.is_filled() {
            let bg_width = self.max_width;
            let bg_height =
                self.background_victoire.height() * (bg_width / self.background_victoire.width());
            draw_texture_ex(
                &self.background_victoire,
                0.,
                0.,
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
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(bg_width, bg_height)),
                    ..Default::default()
                },
            );
        }

        let sudoku_x = mouse_x - self.x_offset;
        let sudoku_y = mouse_y - self.y_offset;
        if sudoku_x < self.grid_size
            && sudoku_x > 0.0
            && sudoku_y < self.grid_size
            && sudoku_y > 0.0
        {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.diag_click(3, x, y);
                // if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y) {
                //     self.selected_cell = None;
                // } else {
                //     self.selected_cell = Some((x, y));
                // }
            }
            self.draw_cell((x, y), Color::from_hex(0xf1f5f9));
        }

        if let Some((x, y)) = self.selected_cell {
            for (x, y) in self.sudoku.get_cell_group(x, y, All) {
                self.draw_cell((x, y), Color::from_hex(0xe4ebf2));
            }
            self.draw_cell((x, y), Color::from_hex(0xc2ddf8));
        }

        //self.draw_simple_sudoku(font.clone(),0, 0).await;
        self.draw_diag_sudoku(font.clone(), 3).await;
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

        if let Some(selected_cell) = &mut self.selected_cell {
            match get_last_key_pressed() {
                Some(KeyCode::Up) => {
                    selected_cell.1 = if selected_cell.1 == 0 {
                        self.sudoku.get_n2() - 1
                    } else {
                        selected_cell.1 - 1
                    };
                }
                Some(KeyCode::Down) => {
                    selected_cell.1 = (selected_cell.1 + 1) % self.sudoku.get_n2();
                }
                Some(KeyCode::Left) => {
                    selected_cell.0 = if selected_cell.0 == 0 {
                        self.sudoku.get_n2() - 1
                    } else {
                        selected_cell.0 - 1
                    };
                }
                Some(KeyCode::Right) => {
                    selected_cell.0 = (selected_cell.0 + 1) % self.sudoku.get_n2();
                }
                Some(KeyCode::Kp1) => {
                    if let Some(action) = self.actions_boutons.get("1").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp2) => {
                    if let Some(action) = self.actions_boutons.get("2").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp3) => {
                    if let Some(action) = self.actions_boutons.get("3").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp4) => {
                    if let Some(action) = self.actions_boutons.get("4").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp5) => {
                    if let Some(action) = self.actions_boutons.get("5").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp6) => {
                    if let Some(action) = self.actions_boutons.get("6").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp7) => {
                    if let Some(action) = self.actions_boutons.get("7").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp8) => {
                    if let Some(action) = self.actions_boutons.get("8").cloned() {
                        action(self);
                    }
                }
                Some(KeyCode::Kp9) => {
                    if let Some(action) = self.actions_boutons.get("9").cloned() {
                        action(self);
                    }
                }
                _ => (),
            }
        }
        match get_last_key_pressed(){
            Some(KeyCode::N) => {
                if let Some(action) = self.actions_boutons.get("Note").cloned() {
                    action(self);
                }
            }
            Some(KeyCode::F) => {
                if let Some(action) = self.actions_boutons.get("Fill Notes").cloned() {
                    action(self);
                }
            }
            Some(KeyCode::U) => {
                if let Some(action) = self.actions_boutons.get("Undo").cloned() {
                    action(self);
                }
            }
            Some(KeyCode::Escape) => {
                self.selected_cell = None;
            }
            Some(KeyCode::A) => {
                if let Some(action) = self.actions_boutons.get("Analyse").cloned() {
                    action(self);
                }
            }
            Some(KeyCode::P) => {
                if let Some(action) = self.actions_boutons.get("Play").cloned() {
                    action(self);
                }
            }
            Some(KeyCode::S) => {
                if let Some(action) = self.actions_boutons.get("Solve").cloned() {
                    action(self);
                }
            }
            _ => (),
        }
    }
}
