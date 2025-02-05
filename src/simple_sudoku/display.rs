use super::{Sudoku, SudokuDisplay};
use macroquad::prelude::*;
use std::collections::HashSet;

impl<'a> SudokuDisplay<'a> {
    pub fn new(sudoku: &'a mut Sudoku) -> Self {
        let scale_factor = 0.8;
        let grid_size = 900.0 * scale_factor;
        let pixel_per_cell = (grid_size / sudoku.get_n2() as f32) * scale_factor;
        let x_offset = 250.0 * scale_factor;
        let y_offset = 50.0 * scale_factor;
        let bx_offset = 150.0 * scale_factor;
        let solvex_offset = 50.0 * scale_factor;
        let solving = false;

        Self {
            sudoku,
            scale_factor,
            grid_size,
            pixel_per_cell,
            selected_cell: None,
            selected_buttons: HashSet::new(),
            x_offset,
            y_offset,
            bx_offset,
            solvex_offset,
            solving,
        }
    }

    pub fn solve_once(&mut self){
        if self.sudoku.solve_once().unwrap()>2 as usize{
            self.solve_once();
        }
    }

    async fn draw_solve(&mut self, font: Font){
        let mut color: Color;
        let solve_sizex = 150.0;
        let solve_sizey = 100.0;
        let solve_ypadding = 10.0;
        let solve1_x = self.x_offset - self.solvex_offset - solve_sizex;
        let solve1_y = self.y_offset + (self.grid_size - (solve_sizey)*2.0 - solve_ypadding)/2.0;
        color = Color::from_hex(0xe4ebf2);

        draw_rectangle(
            solve1_x,
            solve1_y,
            solve_sizex,
            solve_sizey,
            color
        );

        if self.solving{
            color = Color::from_hex(0xc2ddf8);
        }

        let solve2_x = self.x_offset - self.solvex_offset - solve_sizex;
        let solve2_y = self.y_offset + (self.grid_size - (solve_sizey)*2.0 - solve_ypadding)/2.0 + solve_sizey + solve_ypadding;

        draw_rectangle(
            solve2_x,
            solve2_y,
            solve_sizex,
            solve_sizey,
            color
        );

        let font_size = solve_sizey as u16 * 2 / 8;
        let text1 = "solve once";
        let text1_dimensions = measure_text(&text1, Some(&font), font_size, 1.0);
        let text1_x = solve1_x + (solve_sizex - text1_dimensions.width) / 2.0;
        let text1_y = solve1_y + (solve_sizey + text1_dimensions.height) / 2.0;

        draw_text_ex(
            &text1,
            text1_x,
            text1_y,
            TextParams {
                font: Some(&font),
                font_size: font_size,
                color: Color::from_hex(0x000000),
                ..Default::default()
            },
        );

        let text2 = "solve";
        let text2_dimensions = measure_text(&text2, Some(&font), font_size, 1.0);
        let text2_x = solve2_x + (solve_sizex - text2_dimensions.width) / 2.0;
        let text2_y = solve2_y + (solve_sizey + text2_dimensions.height) / 2.0;

        draw_text_ex(
            &text2,
            text2_x,
            text2_y,
            TextParams {
                font: Some(&font),
                font_size: font_size,
                color: Color::from_hex(0x000000),
                ..Default::default()
            },
        );
    }

    async fn draw_buttons(&self, font: Font) {
        let mut color: Color;
        for x in 0..self.sudoku.get_n() {
            for y in 0..self.sudoku.get_n() {
                let b_size = self.pixel_per_cell * 3.0 / 2.0;
                let b_padding = 10.0;
                let b_x = self.x_offset
                    + self.grid_size
                    + self.bx_offset
                    + (x as f32) * (b_size + b_padding);
                let b_y = self.y_offset
                    + (self.grid_size - (b_size + b_padding) * (self.sudoku.get_n() as f32)) / 2.0
                    + (y as f32) * (b_size + b_padding);
                if self.selected_buttons.contains(&(x, y)) {
                    color = Color::from_hex(0xc2ddf8);
                } else {
                    color = Color::from_hex(0xe4ebf2);
                }
                draw_rectangle(b_x, b_y, b_size, b_size, color);
                let font_size = b_size as u16 * 2 / 3;
                let text = (y * self.sudoku.get_n() + x + 1).to_string();
                let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
                let text_x = b_x + (b_size - text_dimensions.width) / 2.0;
                let text_y = b_y + (b_size + text_dimensions.height) / 2.0;
                draw_text_ex(
                    &text,
                    text_x,
                    text_y,
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

        let pb = self.sudoku.get_possibility_board();
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
        self.scale_factor = screen_height() / self.grid_size;
        self.grid_size = screen_height();
        self.pixel_per_cell = (self.grid_size / self.sudoku.get_n2() as f32) * self.scale_factor;
        self.x_offset = 250.0 * self.scale_factor;
        self.y_offset = 0.0 * self.scale_factor;
        self.bx_offset = 50.0 * self.scale_factor;
    }

    pub async fn run(&mut self, font: Font) {
        self.update_scale();
        let b_size = self.pixel_per_cell * 3.0 / 2.0;
        let b_padding = 10.0;

        let (mouse_x, mouse_y) = (
            mouse_position().0 - self.x_offset,
            mouse_position().1 - self.y_offset,
        );
        let x = (mouse_x / self.pixel_per_cell).floor() as usize;
        let y = (mouse_y / self.pixel_per_cell).floor() as usize;

        clear_background(Color::from_hex(0xffffff));

        //si on clique
        if is_mouse_button_pressed(MouseButton::Left) {
            let b_x = self.x_offset + self.grid_size + self.bx_offset;
            let b_y = self.y_offset
                + (self.grid_size - (b_size + b_padding) * (self.sudoku.get_n() as f32)) / 2.0;
            if mouse_x + self.x_offset > b_x
                && mouse_x + self.x_offset
                    < b_x + (b_size + b_padding) * (self.sudoku.get_n() as f32)
                && mouse_y + self.y_offset > b_y
                && mouse_y + self.y_offset
                    < b_y + (b_size + b_padding) * (self.sudoku.get_n() as f32)
            {
                let button = (
                    ((mouse_x + self.x_offset - b_x) / (b_size + b_padding) as f32).floor()
                        as usize,
                    ((mouse_y + self.y_offset - b_y) / (b_size + b_padding) as f32).floor()
                        as usize,
                );
                if self.selected_buttons.contains(&button) {
                    self.selected_buttons.remove(&button);
                } else {
                    self.selected_buttons.insert(button);
                }
            }

            let solve_sizex = 150.0;
            let solve_sizey = 100.0;
            let solve_ypadding = 10.0;

            let solve1_x = self.x_offset - self.solvex_offset - solve_sizex;
            let solve1_y = self.y_offset + (self.grid_size - (solve_sizey)*2.0 - solve_ypadding)/2.0;

            let solve2_x = self.x_offset - self.solvex_offset - solve_sizex;
            let solve2_y = self.y_offset + (self.grid_size - (solve_sizey)*2.0 - solve_ypadding)/2.0 + solve_sizey + solve_ypadding;
            
            if mouse_x + self.x_offset > solve1_x && mouse_y + self.y_offset > solve1_y
            && mouse_x + self.x_offset < solve1_x + solve_sizex && mouse_y + self.y_offset < solve1_y + solve_sizey{
                self.solve_once();
            }

            if mouse_x + self.x_offset > solve2_x && mouse_y + self.y_offset > solve2_y
            && mouse_x + self.x_offset < solve2_x + solve_sizex && mouse_y + self.y_offset < solve2_y + solve_sizey{
                if self.solving{
                    self.solving = false;
                }
                else{
                    self.solving = true;
                }
            }
        }

        //si on clique dans le sudoku
        if (mouse_x as f32) < self.grid_size && (mouse_x as f32) > 0.0 && (mouse_y as f32) < self.grid_size && (mouse_y as f32) > 0.0{

            if is_mouse_button_pressed(MouseButton::Left) {
                if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y) {
                    self.selected_cell = None;
                } else {
                    self.selected_cell = Some((x, y));
                }
                self.selected_buttons.clear();

                if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y){
                    let pb: &HashSet<usize> = &self.sudoku.get_possibility_board()[y][x];
                    for n in pb{
                        for i in 0..self.sudoku.get_n(){
                            for j in 0..self.sudoku.get_n(){
                                if self.sudoku.get_n()*j + i + 1 == *n{
                                    self.selected_buttons.insert((i,j));
                                }
                            }
                        }
                    }
                }
            }
            self.draw_cell(x, y, Color::from_hex(0xf1f5f9));
        }

        if let Some((x, y)) = self.selected_cell {
            for (x, y) in Sudoku::get_cell_groups(self.sudoku.get_n(), x, y)
                .iter()
                .flatten()
            {
                self.draw_cell(*x, *y, Color::from_hex(0xe4ebf2));
            }
            self.draw_cell(x, y, Color::from_hex(0xc2ddf8));
        }

        self.draw_sudoku(font.clone()).await;
        self.draw_buttons(font.clone()).await;
        self.draw_solve(font.clone()).await;
    }
}
