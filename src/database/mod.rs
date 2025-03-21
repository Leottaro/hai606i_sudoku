use diesel::MysqlConnection;
use schema::simple_sudokus;

use crate::simple_sudoku::Sudoku as SimpleSudoku;

pub mod db;
pub mod schema;

pub struct Database {
    connection: MysqlConnection,
}

#[derive(Queryable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct DBSimpleSudoku {
    pub id: i32,
    pub n: u8,
    pub board: Vec<u8>,
    pub difficulty: u8,
}

impl DBSimpleSudoku {
    pub fn to_sudoku(&self) -> SimpleSudoku {
        let mut sudoku = SimpleSudoku::new(self.n as usize);
        let mut board_iter = self.board.clone().into_iter().map(|cell| cell as usize);
        for y in 0..sudoku.get_n2() {
            for x in 0..sudoku.get_n2() {
                let next_value = board_iter.next().unwrap();
                if next_value > 0 {
                    sudoku.set_value(x, y, next_value);
                }
            }
        }
        sudoku
    }
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
