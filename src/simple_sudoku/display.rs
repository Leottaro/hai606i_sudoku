use super::{Button, Sudoku, SudokuDisplay, SudokuGroups::*};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[allow(dead_code)] // no warning due to unused functions
impl<'a> SudokuDisplay<'a> {
    pub async fn new(sudoku: &'a mut Sudoku, font: Font) -> Self {
        let max_height = screen_height();
        let max_width = screen_width();
        let scale_factor = 1.0;
        let grid_size = 900.0 * scale_factor;
        let pixel_per_cell = grid_size / sudoku.get_n2() as f32;
        let x_offset = 250.0 * scale_factor;
        let y_offset = 150.0 * scale_factor;
        let bx_offset = 150.0 * scale_factor;
        let solvex_offset = 50.0 * scale_factor;
        let choosey_offset = (y_offset - 100.0) / 2.0;
        let mode = "play".to_string();
        let player_pboard: Vec<Vec<HashSet<usize>>> =
            vec![vec![HashSet::new(); sudoku.get_n2()]; sudoku.get_n2()];
        let note = false;
        let mut button_list: Vec<Button> = Vec::new();
        let mut actions_boutons: HashMap<String, Rc<Box<dyn Fn(&mut SudokuDisplay) -> ()>>> =
            HashMap::new();
        let background = load_texture("./res/bg/ow-bg.png").await.unwrap();

        // ================== Buttons ==================
        let choose_sizex = 150.0 * scale_factor;
        let choose_sizey = 100.0 * scale_factor;
        let choose_xpadding = 10.0 * scale_factor;

        let bouton_play = Button::new(
            x_offset + (grid_size - choose_sizex * 2.0 - choose_xpadding) / 2.0,
            y_offset - choosey_offset - choose_sizey,
            choose_sizex,
            choose_sizey,
            true,
            "Play".to_string(),
            true,
            scale_factor,
        );

        actions_boutons.insert(
            bouton_play.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode("play".to_string());
                for bouton in sudoku_display.button_list.iter_mut() {
                    if bouton.text == "Analyse".to_string() {
                        bouton.set_clicked(false);
                    }
                    if bouton.text == "Play".to_string() {
                        bouton.set_clicked(true);
                    }
                }
                for i in 1..=sudoku_display.sudoku.get_n2() {
                    for button in sudoku_display.button_list.iter_mut() {
                        if button.text == i.to_string() {
                            button.set_enabled(true);
                            if let Some((x, y)) = sudoku_display.selected_cell {
                                if sudoku_display.player_pboard[y][x].contains(&i) {
                                    button.set_clicked(true);
                                } else {
                                    button.set_clicked(false);
                                }
                            }
                        }
                    }
                }
                for button in sudoku_display.button_list.iter_mut() {
                    if button.text == "Note".to_string() || button.text == "Undo".to_string() {
                        button.set_enabled(true);
                    }
                }
            })),
        );

        button_list.push(bouton_play);

        let button_analyse: Button = Button::new(
            x_offset
                + (grid_size - choose_sizex * 2.0 - choose_xpadding) / 2.0
                + choose_sizex
                + choose_xpadding,
            y_offset - choosey_offset - choose_sizey,
            choose_sizex,
            choose_sizey,
            true,
            "Analyse".to_string(),
            false,
            scale_factor,
        );

        actions_boutons.insert(
            button_analyse.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.set_mode("Analyse".to_string());
                for bouton in sudoku_display.button_list.iter_mut() {
                    if bouton.text == "Play".to_string() {
                        bouton.set_clicked(false);
                    }
                    if bouton.text == "Analyse".to_string() {
                        bouton.set_clicked(true);
                    }
                }
                for i in 1..=sudoku_display.sudoku.get_n2() {
                    for button in sudoku_display.button_list.iter_mut() {
                        if button.text == i.to_string() {
                            button.set_enabled(false);
                        }
                    }
                }
                for button in sudoku_display.button_list.iter_mut() {
                    if button.text == "Note".to_string() || button.text == "Undo".to_string() {
                        button.set_enabled(false);
                    }
                }
            })),
        );

        button_list.push(button_analyse);

        let solve_sizex = 150.0 * scale_factor;
        let solve_sizey = 100.0 * scale_factor;
        let solve_ypadding = 10.0 * scale_factor;
        let solve1_x = x_offset - solvex_offset - solve_sizex;
        let solve1_y = y_offset + (grid_size - solve_sizey * 2.0 - solve_ypadding) / 2.0;

        let button_solve_once = Button::new(
            solve1_x,
            solve1_y,
            solve_sizex,
            solve_sizey,
            true,
            "Solve once".to_string(),
            false,
            scale_factor,
        );

        actions_boutons.insert(
            button_solve_once.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.solve_once();
            })),
        );

        button_list.push(button_solve_once);

        let solve2_x = x_offset - solvex_offset - solve_sizex;
        let solve2_y = y_offset
            + (grid_size - (solve_sizey) * 2.0 - solve_ypadding) / 2.0
            + solve_sizey
            + solve_ypadding;
        let button_solve = Button::new(
            solve2_x,
            solve2_y,
            solve_sizex,
            solve_sizey,
            true,
            "Solve".to_string(),
            false,
            scale_factor,
        );
        actions_boutons.insert(
            "Solve".to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.solve_once();
            })),
        );

        button_list.push(button_solve);

        let b_size = pixel_per_cell * 3.0 / 2.0;
        let b_padding = 10.0;
        let button_note = Button::new(
            x_offset + grid_size + bx_offset,
            y_offset + (grid_size - (b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0
                - solve_sizey
                - solve_ypadding,
            b_size * 1.5 + b_padding * 0.5,
            solve_sizey,
            true,
            "Note".to_string(),
            false,
            scale_factor,
        );

        actions_boutons.insert(
            button_note.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.note = !sudoku_display.note;
                for bouton in sudoku_display.button_list.iter_mut() {
                    if bouton.text == "Note".to_string() {
                        bouton.set_clicked(!bouton.clicked());
                    }
                }
            })),
        );

        button_list.push(button_note);

        let button_undo = Button::new(
            x_offset + grid_size + bx_offset + 1.5 * b_size + b_padding * 1.5,
            y_offset + (grid_size - (b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0
                - solve_sizey
                - solve_ypadding,
            b_size * 1.5 + b_padding * 0.5,
            solve_sizey,
            true,
            "Undo".to_string(),
            false,
            scale_factor,
        );

        actions_boutons.insert(
            button_undo.text.to_string(),
            Rc::new(Box::new(|sudoku_display| {
                sudoku_display.note = !sudoku_display.note;
                for bouton in sudoku_display.button_list.iter_mut() {
                    if bouton.text == "Note".to_string() {
                        bouton.set_clicked(!bouton.clicked());
                    }
                }
            })),
        );

        button_list.push(button_undo);

        for x in 0..sudoku.get_n() {
            for y in 0..sudoku.get_n() {
                let value1 = y * sudoku.get_n() + x + 1;

                let b_x = x_offset + grid_size + bx_offset + (x as f32) * (b_size + b_padding);
                let b_y = y_offset
                    + (grid_size - (b_size + b_padding) * (sudoku.get_n() as f32)) / 2.0
                    + (y as f32) * (b_size + b_padding);
                let bouton_numero = Button::new(
                    b_x,
                    b_y,
                    b_size,
                    b_size,
                    true,
                    value1.to_string(),
                    false,
                    scale_factor,
                );

                actions_boutons.insert(
                    value1.to_string(),
                    Rc::new(Box::new(move |sudoku_display| {
                        if sudoku_display.selected_cell.is_some() {
                            let (x1, y1) = sudoku_display.selected_cell.unwrap();
                            let value = y * sudoku_display.sudoku.get_n() + x + 1;
                            if sudoku_display.note && sudoku_display.sudoku.get_board()[y1][x1] == 0
                            {
                                for bouton in sudoku_display.button_list.iter_mut() {
                                    if bouton.text == value.to_string() {
                                        if bouton.clicked == true {
                                            bouton.set_clicked(false);
                                            sudoku_display.player_pboard[y1][x1].remove(&value);
                                        } else {
                                            bouton.set_clicked(true);
                                            sudoku_display.player_pboard[y1][x1].insert(value);
                                        }
                                    }
                                }
                            } else if !sudoku_display.note {
                                sudoku_display.sudoku.set_value(x1, y1, value);
                                sudoku_display.player_pboard[y1][x1].clear();
                                for n in 1..=sudoku_display.sudoku.get_n2() {
                                    for button in sudoku_display.button_list.iter_mut() {
                                        if button.text == n.to_string() {
                                            button.set_clickable(false);
                                        }
                                    }
                                }
                            }
                        }
                    })),
                );

                button_list.push(bouton_numero);
            }
        }
        // =============================================

        // ================== Actions ==================

        // =============================================

        Self {
            sudoku,
            max_height,
            max_width,
            scale_factor,
            grid_size,
            pixel_per_cell,
            selected_cell: None,
            x_offset,
            y_offset,
            bx_offset,
            solvex_offset,
            mode,
            player_pboard,
            note,
            button_list,
            font,
            actions_boutons,
            background,
        }
    }

    pub fn set_mode(&mut self, mode: String) {
        self.mode = mode;
    }

    pub fn button_list(&self) -> &Vec<Button> {
        &self.button_list
    }

    pub fn solve_once(&mut self) {
        loop {
            match self.sudoku.rule_solve(None) {
                Ok(0 | 1 | 2) => break,
                Ok(_) => (),
                Err(((x1, y1), (x2, y2))) => eprintln!("Error: {x1},{y1} == {x2},{y2}"),
            }
        }
    }

    fn draw_cell(&self, x: usize, y: usize, color: Color) {
        draw_rectangle(
            x as f32 * self.pixel_per_cell + self.x_offset,
            y as f32 * self.pixel_per_cell + self.y_offset,
            self.pixel_per_cell,
            self.pixel_per_cell,
            color,
        );
    }

    async fn draw_sudoku(&self, font: Font) {
        let n = self.sudoku.get_n();
        let n2 = self.sudoku.get_n2();
        for i in 0..n2 {
            let i = i as f32;
            // row
            draw_line(
                0.0 + self.x_offset,
                i * self.pixel_per_cell + self.y_offset,
                self.grid_size + self.x_offset,
                i * self.pixel_per_cell + self.y_offset,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
            // col
            draw_line(
                i * self.pixel_per_cell + self.x_offset,
                0.0 + self.y_offset,
                i * self.pixel_per_cell + self.x_offset,
                self.grid_size + self.y_offset,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
        }

        for y in 0..n {
            for x in 0..n {
                draw_rectangle_lines(
                    (x * n) as f32 * self.pixel_per_cell + self.x_offset,
                    (y * n) as f32 * self.pixel_per_cell + self.y_offset,
                    n as f32 * self.pixel_per_cell,
                    n as f32 * self.pixel_per_cell,
                    2.0,
                    Color::from_hex(0x000000),
                );
            }
        }

        for (y, line) in self.sudoku.get_board().into_iter().enumerate() {
            for (x, cell) in line.into_iter().enumerate() {
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
                    text_x + self.x_offset,
                    text_y + self.y_offset,
                    TextParams {
                        font: Some(&font),
                        font_size: font_size,
                        color: Color::from_hex(0x000000),
                        ..Default::default()
                    },
                );
            }
        }

        let mut pb = self.sudoku.get_possibility_board();
        if self.mode == "play".to_string() {
            pb = self.player_pboard.clone();
        }
        for x in 0..n2 {
            for y in 0..n2 {
                if pb[y][x].len() == 0 {
                    continue;
                }
                let font_size = self.pixel_per_cell as u16 * 2 / (3 * n as u16);
                for i in 0..n {
                    for j in 0..n {
                        let number = i * n + j + 1;
                        if !pb[y][x].contains(&number) {
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
                            text_x + self.x_offset,
                            text_y + self.y_offset,
                            TextParams {
                                font: Some(&font),
                                font_size: font_size,
                                color: Color::from_hex(0x000000),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn update_scale(&mut self) {
        //largeur voulue : 1700
        //hauteur voulue : 1025
        let ratio = screen_height() / screen_width();
        let ratio_voulu = 1025./1700.;
        if ratio>ratio_voulu {
            self.scale_factor = screen_width() / self.max_width;
        }
        else{
            self.scale_factor = screen_height() / self.max_height;
        }
        
        self.grid_size = 900.0 * self.scale_factor;
        self.pixel_per_cell = self.grid_size / self.sudoku.get_n2() as f32;
        self.x_offset = 250.0 * self.scale_factor;
        self.y_offset = 150.0 * self.scale_factor;
        self.bx_offset = 50.0 * self.scale_factor;
        self.solvex_offset = 50.0 * self.scale_factor;
    }

    pub async fn run(&mut self, font: Font) {
        self.update_scale();

        let (mouse_x, mouse_y) = (mouse_position().0, mouse_position().1);
        let x = ((mouse_x - self.x_offset) / self.pixel_per_cell).floor() as usize;
        let y = ((mouse_y - self.y_offset) / self.pixel_per_cell).floor() as usize;

        //test bg

        clear_background(Color::from_hex(0xffffff));

        
        draw_texture(&self.background,0.,0.,WHITE);
        

        //si on clique dans le sudoku
        let sudoku_x = mouse_x - self.x_offset;
        let sudoku_y = mouse_y - self.y_offset;
        if (sudoku_x as f32) < self.grid_size
            && (sudoku_x as f32) > 0.0
            && (sudoku_y as f32) < self.grid_size
            && (sudoku_y as f32) > 0.0
        {
            if is_mouse_button_pressed(MouseButton::Left) {
                if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y) {
                    self.selected_cell = None;
                } else {
                    self.selected_cell = Some((x, y));
                }
                for n in 1..=self.sudoku.get_n2() {
                    for button in self.button_list.iter_mut() {
                        if button.text == n.to_string() {
                            button.set_clicked(false);
                        }
                    }
                }

                if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y) {
                    let mut pb: &HashSet<usize> = &self.sudoku.get_possibility_board()[y][x];

                    if self.mode == "play".to_string() {
                        pb = &self.player_pboard[y][x];
                    }

                    for n in pb {
                        for button in self.button_list.iter_mut() {
                            if button.text == n.to_string() {
                                button.set_clicked(true);
                            }
                        }
                    }

                    if self.sudoku.get_board()[y][x] != 0 {
                        for n in 1..=self.sudoku.get_n2() {
                            for button in self.button_list.iter_mut() {
                                if button.text == n.to_string() {
                                    button.set_clickable(false);
                                }
                            }
                        }
                    } else {
                        for n in 1..=self.sudoku.get_n2() {
                            for button in self.button_list.iter_mut() {
                                if button.text == n.to_string() {
                                    button.set_clickable(true);
                                }
                            }
                        }
                    }
                }
            }
            self.draw_cell(x, y, Color::from_hex(0xf1f5f9));
        }

        if let Some((x, y)) = self.selected_cell {
            for (x, y) in self.sudoku.get_cell_group(x, y, ALL) {
                self.draw_cell(x, y, Color::from_hex(0xe4ebf2));
            }
            self.draw_cell(x, y, Color::from_hex(0xc2ddf8));
        }

        self.draw_sudoku(font.clone()).await;
        let mut action: Option<Rc<Box<dyn Fn(&mut SudokuDisplay)>>> = None;
        for bouton in self.button_list.iter_mut() {
            bouton.set_scale_factor(self.scale_factor);
            if !bouton.enabled() {
                continue;
            }
            if mouse_x > bouton.x()
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
        //self.draw_buttons(font.clone()).await;
        //self.draw_solve(font.clone()).await;
        //self.draw_chooser(font.clone()).await;
    }
}
