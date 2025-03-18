use diesel::MysqlConnection;
use schema::simple_sudokus;
use std::sync::Mutex;

use crate::simple_sudoku::Sudoku as SimpleSudoku;

pub mod db;
pub mod schema;

pub struct Database {
    connection: Mutex<MysqlConnection>,
}

#[derive(Queryable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct DBSimpleSudoku {
    pub id: i32,
    pub n: u8,
    pub board: Vec<u8>,
    pub difficulty: u8,
}

#[derive(Insertable)]
#[diesel(table_name = simple_sudokus)]
pub struct DBNewSimpleSudoku {
    pub n: u8,
    pub board: Vec<u8>,
    pub difficulty: u8,
}

impl DBNewSimpleSudoku {
    pub fn from(sudoku: &SimpleSudoku) -> Self {
        Self {
            n: sudoku.get_n() as u8,
            board: sudoku
                .get_board()
                .into_iter()
                .flat_map(|line| line.into_iter().map(|cell| cell as u8))
                .collect(),
            difficulty: sudoku.get_difficulty() as u8,
        }
    }
}
