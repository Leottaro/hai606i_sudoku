use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};

use crate::simple_sudoku::{Sudoku as SimpleSudoku, SudokuDifficulty};

use super::{
    schema::{
        simple_sudoku_canonical::dsl::*, simple_sudoku_canonical_squares::dsl::*,
        simple_sudoku_games::dsl::*,
    },
    DBNewSimpleSudokuGame, DBSimpleSudokuCanonical, DBSimpleSudokuCanonicalSquares,
    DBSimpleSudokuGame, Database,
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

    //////////////////////////////////////////////////////////////////
    // GET ALL

    pub fn get_all_simple_sudoku_canonical(
        &mut self,
    ) -> Result<Vec<DBSimpleSudokuCanonical>, diesel::result::Error> {
        simple_sudoku_canonical.get_results::<DBSimpleSudokuCanonical>(&mut self.connection)
    }

    pub fn get_all_simple_sudoku_canonical_squares(
        &mut self,
    ) -> Result<Vec<DBSimpleSudokuCanonicalSquares>, diesel::result::Error> {
        simple_sudoku_canonical_squares
            .get_results::<DBSimpleSudokuCanonicalSquares>(&mut self.connection)
    }

    pub fn get_all_simple_sudoku_game(
        &mut self,
    ) -> Result<Vec<DBSimpleSudokuGame>, diesel::result::Error> {
        simple_sudoku_games.get_results::<DBSimpleSudokuGame>(&mut self.connection)
    }

    //////////////////////////////////////////////////////////////////
    // GET N

    pub fn get_n_simple_sudoku_canonical(
        &mut self,
        n: i64,
    ) -> Result<Vec<DBSimpleSudokuCanonical>, diesel::result::Error> {
        simple_sudoku_canonical
            .limit(n)
            .get_results::<DBSimpleSudokuCanonical>(&mut self.connection)
    }

    pub fn get_n_simple_sudoku_canonical_squares(
        &mut self,
        n: i64,
    ) -> Result<Vec<DBSimpleSudokuCanonicalSquares>, diesel::result::Error> {
        simple_sudoku_canonical_squares
            .limit(n)
            .get_results::<DBSimpleSudokuCanonicalSquares>(&mut self.connection)
    }

    pub fn get_n_simple_sudoku_game(
        &mut self,
        n: i64,
    ) -> Result<Vec<DBSimpleSudokuGame>, diesel::result::Error> {
        simple_sudoku_games
            .limit(n)
            .get_results::<DBSimpleSudokuGame>(&mut self.connection)
    }

    //////////////////////////////////////////////////////////////////
    // INSERT

    pub fn insert_simple_sudoku_canonical(
        &mut self,
        sudoku: DBSimpleSudokuCanonical,
        squares: Vec<DBSimpleSudokuCanonicalSquares>,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(simple_sudoku_canonical)
            .values(&sudoku)
            .execute(&mut self.connection)?;
        diesel::insert_into(simple_sudoku_canonical_squares)
            .values(&squares)
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

    //////////////////////////////////////////////////////////////////
    // INSERT MULTIPLE

    pub fn insert_multiple_simple_sudoku_canonical(
        &mut self,
        sudokus: Vec<DBSimpleSudokuCanonical>,
        squares: Vec<DBSimpleSudokuCanonicalSquares>,
    ) -> Result<(usize, usize), diesel::result::Error> {
        let mut inserted_sudokus = 0;
        let mut inserted_squares = 0;

        for sudokus_chunk in sudokus.chunks(16348) {
            inserted_sudokus += diesel::insert_into(simple_sudoku_canonical)
                .values(sudokus_chunk)
                .execute(&mut self.connection)?;
        }

        for squares_chunk in squares.chunks(16348) {
            inserted_squares += diesel::insert_into(simple_sudoku_canonical_squares)
                .values(squares_chunk)
                .execute(&mut self.connection)?;
        }

        Ok((inserted_sudokus, inserted_squares))
    }

    pub fn insert_multiple_simple_sudoku_game(
        &mut self,
        sudokus: Vec<DBNewSimpleSudokuGame>,
    ) -> Result<usize, diesel::result::Error> {
        let mut inserted_sudokus = 0;
        for sudokus_chunk in sudokus.chunks(16348) {
            inserted_sudokus += diesel::insert_into(simple_sudoku_games)
                .values(sudokus_chunk)
                .execute(&mut self.connection)?;
        }
        Ok(inserted_sudokus)
    }

    //////////////////////////////////////////////////////////////////
    // GET RANDOM

    pub fn get_random_simple_sudoku_canonical(
        &mut self,
        n: u8,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = simple_sudoku_canonical
            .filter(sudoku_n.eq(n))
            .count()
            .get_result::<i64>(&mut self.connection)?;
        simple_sudoku_canonical
            .filter(sudoku_n.eq(n))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBSimpleSudokuCanonical>(&mut self.connection)
            .map(SimpleSudoku::db_from_canonical)
    }

    pub fn get_random_simple_sudoku_game(
        &mut self,
        n: u8,
        difficulty: SudokuDifficulty,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = simple_sudoku_games
            .filter(game_n.eq(n).and(game_difficulty.eq(difficulty as u8)))
            .count()
            .get_result::<i64>(&mut self.connection)?;

        simple_sudoku_games
            .filter(game_n.eq(n).and(game_difficulty.eq(difficulty as u8)))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBSimpleSudokuGame>(&mut self.connection)
            .map(SimpleSudoku::db_from_game)
    }
}
