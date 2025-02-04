use super::{Sudoku, SudokuDisplay};
use macroquad::prelude::*;
use std::collections::HashSet;

impl<'a> SudokuDisplay<'a> {
    pub fn new(sudoku: &'a mut Sudoku) -> Self {
        let window_size = 900.0;
        let pixel_per_cell = window_size / sudoku.get_n2() as f32;
        let x_offset = 250.0;
        let y_offset = 50.0;
        let bx_offset = 150.0;
        let solving = false;

        Self {
            sudoku,
            window_size,
            pixel_per_cell,
            selected_cell: None,
            selected_buttons: HashSet::new(),
            x_offset,
            y_offset,
            bx_offset,
            solving,
        }
    }

    pub fn solve_once(&mut self){
        self.sudoku.solve_once();
    }

    async fn draw_buttons(&self, font: Font){
        
        let mut color: Color;
        for x in 0..self.sudoku.get_n(){
            for y in 0..self.sudoku.get_n(){
                let b_size = 150.0;
                let b_padding = 10.0;
                let b_x = self.x_offset + self.window_size + self.bx_offset + (x as f32) * (b_size + b_padding);
                let b_y = self.y_offset + (self.window_size - (b_size + b_padding)*(self.sudoku.get_n() as f32))/2.0 + (y as f32) * (b_size + b_padding);
                if self.selected_buttons.contains(&(x,y)){
                    color = Color::from_hex(0xc2ddf8);
                }
                else{
                    color = Color::from_hex(0xe4ebf2);
                }
                draw_rectangle(
                    b_x,
                    b_y,
                    b_size,
                    b_size,
                    color
                );
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
                self.window_size + self.x_offset,
                i * self.pixel_per_cell + self.y_offset,
                1.0,
                Color::from_hex(0xc0c5d3),
            );
            // col
            draw_line(
                i * self.pixel_per_cell + self.x_offset,
                0.0 + self.y_offset,
                i * self.pixel_per_cell + self.x_offset,
                self.window_size + self.y_offset,
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
                let text_x = (x as f32 * self.pixel_per_cell) + (self.pixel_per_cell - text_dimensions.width) / 2.0;
                let text_y = (y as f32 * self.pixel_per_cell) + (self.pixel_per_cell + text_dimensions.height) / 2.0;
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
        for x in 0..n2{
            for y in 0..n2{
                if pb[y][x].len()==0{
                    continue;
                }
                let font_size = self.pixel_per_cell as u16 * 2 / (3 * n as u16);
                for i in 0..n{
                    for j in 0..n{
                        let number = i*n + j + 1;
                        if !pb[y][x].contains(&number){
                            continue;
                        }
                        let text = number.to_string();
                        let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
                        let text_x = (x as f32 * self.pixel_per_cell) - (self.pixel_per_cell / n as f32) + ((j as f32 + 1.0) * self.pixel_per_cell / n as f32) + (self.pixel_per_cell / n as f32 - text_dimensions.width) / 2.0;
                        let text_y = (y as f32 * self.pixel_per_cell) - (self.pixel_per_cell / n as f32)+ ((i as f32 + 1.0) * self.pixel_per_cell / n as f32) + (self.pixel_per_cell / n as f32 + text_dimensions.height) / 2.0;
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

    pub async fn run(&mut self, font: Font) {
        let (mouse_x, mouse_y) = (mouse_position().0 - self.x_offset, mouse_position().1 - self.y_offset);
        let x = (mouse_x / self.pixel_per_cell).floor() as usize;
        let y = (mouse_y / self.pixel_per_cell).floor() as usize;

        

        clear_background(Color::from_hex(0xffffff));

        if is_mouse_button_pressed(MouseButton::Left) {
            let b_size = 150.0;
            let b_padding = 10.0;
            let b_x = self.x_offset + self.window_size + self.bx_offset;
            let b_y = self.y_offset + (self.window_size - (b_size + b_padding)*(self.sudoku.get_n() as f32))/2.0;
            if mouse_x + self.x_offset > b_x && mouse_x + self.x_offset < b_x + (b_size + b_padding) * (self.sudoku.get_n() as f32)
            && mouse_y + self.y_offset > b_y && mouse_y + self.y_offset < b_y + (b_size + b_padding) * (self.sudoku.get_n() as f32){
                let button = (((mouse_x + self.x_offset - b_x)/(b_size+b_padding) as f32).floor() as usize, ((mouse_y + self.y_offset - b_y)/(b_size+b_padding) as f32).floor() as usize);
                if self.selected_buttons.contains(&button){
                    self.selected_buttons.remove(&button);
                }
                else{
                    self.selected_buttons.insert(button);
                }
            }
        }

        if (mouse_x as f32) < self.window_size && (mouse_x as f32) > 0.0 && (mouse_y as f32) < self.window_size && (mouse_y as f32) > 0.0{

            if is_mouse_button_pressed(MouseButton::Left) {
                if self.selected_cell.is_some() && self.selected_cell.unwrap() == (x, y) {
                    self.selected_cell = None;
                } else {
                    self.selected_cell = Some((x, y));
                }
                self.solve_once();
            }
            
            self.draw_cell(x, y, Color::from_hex(0xf1f5f9));
            
        }

        if let Some((x, y)) = self.selected_cell {
            for (x, y) in Sudoku::get_cell_groups(self.sudoku.get_n(), x, y).iter().flatten() {
                self.draw_cell(*x, *y, Color::from_hex(0xe4ebf2));
            }
            self.draw_cell(x, y, Color::from_hex(0xc2ddf8));
        }
        

        self.draw_sudoku(font.clone()).await;
        self.draw_buttons(font.clone()).await;
    }
}