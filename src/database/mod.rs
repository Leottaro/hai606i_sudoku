use diesel::MysqlConnection;
use schema::{simple_sudoku_filled, simple_sudoku_games};

use crate::simple_sudoku::Sudoku as SimpleSudoku;

pub mod db;
pub mod schema;

pub struct Database {
    connection: MysqlConnection,
}

#[derive(Insertable, Queryable, Clone)]
#[diesel(table_name = simple_sudoku_filled)]
pub struct DBSimpleSudokuFilled {
    pub filled_board_hash: u64,
    pub filled_n: u8,
    pub filled_board: Vec<u8>,
    pub filled_up_left_corner: u64,
    pub filled_up_right_corner: u64,
    pub filled_bottom_left_corner: u64,
    pub filled_bottom_right_corner: u64,
}

impl DBSimpleSudokuFilled {
    pub fn to_sudoku(&self) -> SimpleSudoku {
        let mut sudoku = SimpleSudoku::new(self.filled_n as usize);
        for y in 0..sudoku.get_n2() {
            for x in 0..sudoku.get_n2() {
                let value = self.filled_board[y * self.filled_n as usize + x] as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value);
                }
            }
        }
        sudoku
    }
}

#[derive(Queryable)]
pub struct DBSimpleSudokuGame {
    pub game_id: i32,
    pub filled_board_hash: u64,
    pub game_n: u8,
    pub game_board: Vec<u8>,
    pub game_difficulty: u8,
}

impl DBSimpleSudokuGame {
    pub fn to_sudoku(&self) -> SimpleSudoku {
        let mut sudoku = SimpleSudoku::new(self.game_n as usize);
        for y in 0..sudoku.get_n2() {
            for x in 0..sudoku.get_n2() {
                let value = self.game_board[y * self.game_n as usize + x] as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value);
                }
            }
        }
        sudoku
    }
}

#[derive(Insertable, Clone)]
#[diesel(table_name = simple_sudoku_games)]
pub struct DBNewSimpleSudokuGame {
    pub filled_board_hash: u64,
    pub game_n: u8,
    pub game_board: Vec<u8>,
    pub game_difficulty: u8,
}
