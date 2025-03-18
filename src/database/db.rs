use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};

use crate::{
    database::DBSimpleSudoku,
    simple_sudoku::{Sudoku as SimpleSudoku, SudokuDifficulty},
};

use super::{schema::simple_sudokus::dsl::*, DBNewSimpleSudoku, Database};

impl Database {
    pub fn connect() -> Option<Self> {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|err| {
            panic!("Couldn't get DATABASE_URL environment variable: {}", err)
        });

        let connection = MysqlConnection::establish(&database_url);
        if let Err(error) = connection {
            eprintln!("Error connecting to db at {database_url}: {}", error);
            None
        } else {
            Some(Self {
                connection: connection.unwrap(),
            })
        }
    }

    pub fn insert_simple_sudoku(
        &mut self,
        sudoku: DBNewSimpleSudoku,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(simple_sudokus)
            .values(&sudoku)
            .execute(&mut self.connection)
    }

    pub fn get_random_simple_sudoku(
        &mut self,
        sudoku_n: usize,
        sudoku_diff: SudokuDifficulty,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        define_sql_function! {
            fn rand() -> Text;
        };
        let nb_max = simple_sudokus
            .filter(n.eq(sudoku_n as u8).and(difficulty.eq(sudoku_diff as u8)))
            .count()
            .get_result::<i64>(&mut self.connection)?;
        simple_sudokus
            .filter(difficulty.eq(sudoku_diff as u8).and(n.eq(sudoku_n as u8)))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBSimpleSudoku>(&mut self.connection)
            .map(|db_simple_sudoku| db_simple_sudoku.to_sudoku())
    }
}
