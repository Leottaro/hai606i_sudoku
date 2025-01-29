
use macroquad::prelude::*;
use macroquad::text::Font;
use std::sync::Arc;
use std::sync::Mutex;

use super::simple_sudoku::Sudoku;

pub struct SudokuDisplay<'a> {
    sudoku: &'a mut Sudoku,
    window_size: f32,
    pixel_per_cell: f32,
    selected_cell: Option<(usize, usize)>,
}

impl<'a> SudokuDisplay<'a> {
    pub fn new(sudoku: &'a mut Sudoku) -> Self {
        let window_size = 900.0;
        let pixel_per_cell = window_size / sudoku.get_n2() as f32;

        Self {
            sudoku,
            window_size,
            pixel_per_cell,
            selected_cell: None,
        }
    }

    fn draw_cell(&self, x: usize, y: usize, color: Color) {
        draw_rectangle(
            x as f32 * self.pixel_per_cell,
            y as f32 * self.pixel_per_cell,
            self.pixel_per_cell,
            self.pixel_per_cell,
            color,
        );
    }

    async fn draw_sudoku(&self) {
        const REGULAR: &[u8] = include_bytes!("../../res/font/regular.ttf");

        // let fonts = Font::load_from_bytes("regular", REGULAR).unwrap();
        // let font = load_file("./res/font/regular.ttf")
        //         .await
        //         .unwrap();
        
        let regular = load_ttf_font_from_bytes(REGULAR);
        for i in 0..self.sudoku.get_n2() {
            let i = i as f32;
            // row
            draw_line(
                0.0,
                i * self.pixel_per_cell,
                self.window_size,
                i * self.pixel_per_cell,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
            // col
            draw_line(
                i * self.pixel_per_cell,
                0.0,
                i * self.pixel_per_cell,
                self.window_size,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
        }

        for y in 0..self.sudoku.get_n() {
            for x in 0..self.sudoku.get_n() {
                draw_rectangle_lines(
                    (x * self.sudoku.get_n()) as f32 * self.pixel_per_cell,
                    (y * self.sudoku.get_n()) as f32 * self.pixel_per_cell,
                    self.sudoku.get_n() as f32 * self.pixel_per_cell,
                    self.sudoku.get_n() as f32 * self.pixel_per_cell,
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
                draw_text_ex(
                    &cell.to_string(),
                    (x as f32 + 0.25) * self.pixel_per_cell,
                    (y as f32 + 1.0 - 0.25) * self.pixel_per_cell,
                    TextParams {
                        font: regular,
                        font_size: self.pixel_per_cell as u16,
                        color: Color::from_hex(0x000000),
                        ..Default::default()
                    },
                );
                let coordx = x as f32 * self.pixel_per_cell;
                let coordy = y as f32 * self.pixel_per_cell;
                
            }
        }
    }

    pub async fn run(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        let x = (mouse_x / 100.0).floor() as usize;
        let y = (mouse_y / 100.0).floor() as usize;

        if is_mouse_button_pressed(MouseButton::Left) {
            if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y) {
                self.selected_cell = None;
            } else {
                self.selected_cell = Some((x, y));
            }
        }

        clear_background(Color::from_hex(0xffffff));

        self.draw_cell(x, y, Color::from_hex(0xf1f5f9));
        if let Some((x, y)) = self.selected_cell {
            for (x, y) in Sudoku::get_cell_groups(3, x, y).iter().flatten() {
                self.draw_cell(*x, *y, Color::from_hex(0xe4ebf2));
            }
            self.draw_cell(x, y, Color::from_hex(0xc2ddf8));
        }

        self.draw_sudoku().await;
    }
}
