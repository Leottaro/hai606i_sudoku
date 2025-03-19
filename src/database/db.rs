use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};

use crate::simple_sudoku::{Sudoku as SimpleSudoku, SudokuDifficulty};

use super::{
    schema::{simple_sudoku_filled::dsl::*, simple_sudoku_games::dsl::*},
    DBNewSimpleSudokuGame, DBSimpleSudokuFilled, DBSimpleSudokuGame, Database,
};

define_sql_function! {
    fn rand() -> Text;
}

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

    pub fn insert_simple_sudoku_filled(
        &mut self,
        sudoku: DBSimpleSudokuFilled,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(simple_sudoku_filled)
            .values(&sudoku)
            .execute(&mut self.connection)
    }

    pub fn insert_multiple_simple_sudoku_filled(
        &mut self,
        sudokus: Vec<DBSimpleSudokuFilled>,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_or_ignore_into(simple_sudoku_filled)
            .values(&sudokus)
            .execute(&mut self.connection)
    }

    pub fn insert_simple_sudoku_game(
        &mut self,
        sudoku: DBNewSimpleSudokuGame,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(simple_sudoku_games)
            .values(&sudoku)
            .execute(&mut self.connection)
    }

    pub fn insert_multiple_simple_sudoku_game(
        &mut self,
        sudokus: Vec<DBNewSimpleSudokuGame>,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(simple_sudoku_games)
            .values(&sudokus)
            .execute(&mut self.connection)
    }

    pub fn get_random_simple_sudoku_filled(
        &mut self,
        n: usize,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = simple_sudoku_filled
            .filter(filled_n.eq(n as u8))
            .count()
            .get_result::<i64>(&mut self.connection)?;
        simple_sudoku_filled
            .filter(filled_n.eq(n as u8))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBSimpleSudokuFilled>(&mut self.connection)
            .map(|db_simple_sudoku| db_simple_sudoku.to_sudoku())
    }

    pub fn get_random_simple_sudoku_game(
        &mut self,
        n: usize,
        difficulty: SudokuDifficulty,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = simple_sudoku_games
            .filter(game_n.eq(n as u8).and(game_difficulty.eq(difficulty as u8)))
            .count()
            .get_result::<i64>(&mut self.connection)?;

        simple_sudoku_games
            .filter(game_n.eq(n as u8).and(game_difficulty.eq(difficulty as u8)))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBSimpleSudokuGame>(&mut self.connection)
            .map(|db_simple_sudoku| db_simple_sudoku.to_sudoku())
    }
}
