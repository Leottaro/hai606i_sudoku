use std::sync::Mutex;

use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};

use crate::{
    database::DBSimpleSudoku,
    simple_sudoku::{Sudoku as SimpleSudoku, SudokuDifficulty},
};

use super::{schema::simple_sudokus::dsl::*, DBNewSimpleSudoku, Database};

impl Database {
    pub fn connect() -> Self {
        println!("reading .env file...");
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|err| {
            panic!("Couldn't get DATABASE_URL environment variable: {}", err)
        });

        println!("connecting to db at {}...", database_url);
        let mysql_connection = MysqlConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Self {
            connection: Mutex::new(mysql_connection),
        }
    }

    pub fn get_connection(&self) -> &Mutex<MysqlConnection> {
        &self.connection
    }

    pub fn insert_simple_sudoku(
        &self,
        sudoku: DBNewSimpleSudoku,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(simple_sudokus)
            .values(&sudoku)
            .execute(&mut *self.connection.lock().unwrap())
    }

    pub fn get_random_simple_sudoku(
        &self,
        sudoku_n: usize,
        sudoku_diff: SudokuDifficulty,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = simple_sudokus
            .filter(difficulty.eq(sudoku_diff as u8).and(n.eq(sudoku_n as u8)))
            .count()
            .execute(&mut *self.connection.lock().unwrap())?;
        simple_sudokus
            .filter(difficulty.eq(sudoku_diff as u8).and(n.eq(sudoku_n as u8)))
            .limit(nb_max as i64 - 1)
            .get_result::<DBSimpleSudoku>(&mut *self.connection.lock().unwrap())
            .map(|db_simple_sudoku| SimpleSudoku::from_db(&db_simple_sudoku))
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::connect()
    }
}
