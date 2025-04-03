use diesel::{
    BoolExpressionMethods,
    Connection,
    ExpressionMethods,
    PgConnection,
    QueryDsl,
    RunQueryDsl,
};

use crate::simple_sudoku::{ Sudoku as SimpleSudoku, SudokuDifficulty };

use super::{
    schema::{
        canonical_sudokus::dsl::*,
        canonical_sudoku_squares::dsl::*,
        canonical_sudoku_games::dsl::*,
    },
    DBNewCanonicalSudokuGame,
    DBCanonicalSudoku,
    DBCanonicalSudokuSquare,
    DBCanonicalSudokuGame,
    Database,
};

define_sql_function! {
    fn rand() -> Text;
}

impl Database {
    pub fn connect() -> Option<Self> {
        dotenv::dotenv().ok();
        let database_url = std::env
            ::var("DATABASE_URL")
            .unwrap_or_else(|err| {
                panic!("Couldn't get DATABASE_URL environment variable: {}", err)
            });

        let connection = PgConnection::establish(&database_url);
        if let Err(error) = connection {
            eprintln!("Error connecting to db at {database_url}: {}", error);
            None
        } else {
            Some(Self {
                connection: connection.unwrap(),
            })
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   GEL ALL   //////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_all_canonical_sudokus(
        &mut self
    ) -> Result<Vec<DBCanonicalSudoku>, diesel::result::Error> {
        canonical_sudokus.get_results::<DBCanonicalSudoku>(&mut self.connection)
    }

    pub fn get_all_canonical_sudoku_squares(
        &mut self
    ) -> Result<Vec<DBCanonicalSudokuSquare>, diesel::result::Error> {
        canonical_sudoku_squares.get_results::<DBCanonicalSudokuSquare>(&mut self.connection)
    }

    pub fn get_all_canonical_sudoku_games(
        &mut self
    ) -> Result<Vec<DBCanonicalSudokuGame>, diesel::result::Error> {
        canonical_sudoku_games.get_results::<DBCanonicalSudokuGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   GEL N   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_n_canonical_sudokus(
        &mut self,
        n: i64
    ) -> Result<Vec<DBCanonicalSudoku>, diesel::result::Error> {
        canonical_sudokus.limit(n).get_results::<DBCanonicalSudoku>(&mut self.connection)
    }

    pub fn get_n_canonical_sudoku_squares(
        &mut self,
        n: i64
    ) -> Result<Vec<DBCanonicalSudokuSquare>, diesel::result::Error> {
        canonical_sudoku_squares
            .limit(n)
            .get_results::<DBCanonicalSudokuSquare>(&mut self.connection)
    }

    pub fn get_n_canonical_sudoku_games(
        &mut self,
        n: i64
    ) -> Result<Vec<DBCanonicalSudokuGame>, diesel::result::Error> {
        canonical_sudoku_games.limit(n).get_results::<DBCanonicalSudokuGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   INSERT   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn insert_canonical_sudoku(
        &mut self,
        sudoku: DBCanonicalSudoku,
        squares: Vec<DBCanonicalSudokuSquare>
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(canonical_sudokus).values(&sudoku).execute(&mut self.connection)?;
        diesel::insert_into(canonical_sudoku_squares).values(&squares).execute(&mut self.connection)
    }

    pub fn insert_canonical_sudoku_game(
        &mut self,
        sudoku: DBNewCanonicalSudokuGame
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(canonical_sudoku_games).values(&sudoku).execute(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////   INSERT MULTIPLE   //////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn insert_multiple_canonical_sudokus(
        &mut self,
        sudokus: Vec<DBCanonicalSudoku>,
        squares: Vec<DBCanonicalSudokuSquare>
    ) -> Result<(usize, usize), diesel::result::Error> {
        let mut inserted_sudokus = 0;
        let mut inserted_squares = 0;

        for sudokus_chunk in sudokus.chunks(16348) {
            inserted_sudokus += diesel
                ::insert_into(canonical_sudokus)
                .values(sudokus_chunk)
                .execute(&mut self.connection)?;
        }

        for squares_chunk in squares.chunks(16348) {
            inserted_squares += diesel
                ::insert_into(canonical_sudoku_squares)
                .values(squares_chunk)
                .execute(&mut self.connection)?;
        }

        Ok((inserted_sudokus, inserted_squares))
    }

    pub fn insert_multiple_canonical_sudoku_game(
        &mut self,
        sudokus: Vec<DBNewCanonicalSudokuGame>
    ) -> Result<usize, diesel::result::Error> {
        let mut inserted_sudokus = 0;
        for sudokus_chunk in sudokus.chunks(16348) {
            inserted_sudokus += diesel
                ::insert_into(canonical_sudoku_games)
                .values(sudokus_chunk)
                .execute(&mut self.connection)?;
        }
        Ok(inserted_sudokus)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////   GEL RANDOM   /////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_random_canonical_sudokus(
        &mut self,
        n: u8
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = canonical_sudokus
            .filter(sudoku_n.eq(n as i16))
            .count()
            .get_result::<i64>(&mut self.connection)?;
        canonical_sudokus
            .filter(sudoku_n.eq(n as i16))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBCanonicalSudoku>(&mut self.connection)
            .map(SimpleSudoku::db_from_canonical)
    }

    pub fn get_random_canonical_sudoku_game(
        &mut self,
        n: u8,
        difficulty: SudokuDifficulty
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = canonical_sudoku_games
            .filter(game_n.eq(n as i16).and(game_difficulty.eq(difficulty as i16)))
            .count()
            .get_result::<i64>(&mut self.connection)?;

        canonical_sudoku_games
            .filter(game_n.eq(n as i16).and(game_difficulty.eq(difficulty as i16)))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBCanonicalSudokuGame>(&mut self.connection)
            .map(SimpleSudoku::db_from_game)
    }
}
