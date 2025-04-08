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
    *,
    schema::{
        canonical_sudokus::dsl::*,
        canonical_sudoku_squares::dsl::*,
        canonical_sudoku_games::dsl::*,
        canonical_carpets::dsl::*,
        canonical_carpet_sudokus::dsl::*,
        canonical_carpet_games::dsl::*,
    },
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

    pub fn get_all_canonical_sudoku_games(
        &mut self
    ) -> Result<Vec<DBCanonicalSudokuGame>, diesel::result::Error> {
        canonical_sudoku_games.get_results::<DBCanonicalSudokuGame>(&mut self.connection)
    }

    pub fn get_all_canonical_carpets(
        &mut self
    ) -> Result<Vec<DBCanonicalCarpet>, diesel::result::Error> {
        canonical_carpets.get_results::<DBCanonicalCarpet>(&mut self.connection)
    }

    pub fn get_all_canonical_carpet_games(
        &mut self
    ) -> Result<Vec<DBCanonicalCarpetGame>, diesel::result::Error> {
        canonical_carpet_games.get_results::<DBCanonicalCarpetGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   GEL N   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_n_canonical_sudokus(
        &mut self,
        n: i64
    ) -> Result<Vec<DBCanonicalSudoku>, diesel::result::Error> {
        canonical_sudokus
            .order(rand())
            .limit(n)
            .get_results::<DBCanonicalSudoku>(&mut self.connection)
    }

    pub fn get_n_canonical_sudoku_games(
        &mut self,
        n: i64
    ) -> Result<Vec<DBCanonicalSudokuGame>, diesel::result::Error> {
        canonical_sudoku_games
            .order(rand())
            .limit(n)
            .get_results::<DBCanonicalSudokuGame>(&mut self.connection)
    }

    pub fn get_n_canonical_carpets(
        &mut self,
        n: i64
    ) -> Result<Vec<DBCanonicalCarpet>, diesel::result::Error> {
        canonical_carpets
            .order(rand())
            .limit(n)
            .get_results::<DBCanonicalCarpet>(&mut self.connection)
    }

    pub fn get_n_canonical_carpet_games(
        &mut self,
        n: i64
    ) -> Result<Vec<DBCanonicalCarpetGame>, diesel::result::Error> {
        canonical_carpet_games
            .order(rand())
            .limit(n)
            .get_results::<DBCanonicalCarpetGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   INSERT   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn insert_canonical_sudoku(
        &mut self,
        sudoku: DBCanonicalSudoku,
        squares: Vec<DBCanonicalSudokuSquare>
    ) -> Result<usize, diesel::result::Error> {
        Ok(
            diesel::insert_into(canonical_sudokus).values(&sudoku).execute(&mut self.connection)? +
                diesel
                    ::insert_into(canonical_sudoku_squares)
                    .values(&squares)
                    .execute(&mut self.connection)?
        )
    }

    pub fn insert_canonical_sudoku_game(
        &mut self,
        sudoku: DBNewCanonicalSudokuGame
    ) -> Result<DBCanonicalSudokuGame, diesel::result::Error> {
        diesel::insert_into(canonical_sudoku_games).values(&sudoku).get_result(&mut self.connection)
    }

    pub fn insert_canonical_carpet(
        &mut self,
        carpet: DBCanonicalCarpet,
        sudokus: Vec<DBCanonicalCarpetSudoku>
    ) -> Result<usize, diesel::result::Error> {
        diesel
            ::insert_into(canonical_carpets)
            .values(&carpet)
            .get_result::<DBCanonicalCarpet>(&mut self.connection)?;
        let inserted_sudokus_count = self.insert_multiple_canonical_carpet_sudokus(sudokus)?;

        Ok(1 + inserted_sudokus_count)
    }

    pub fn insert_canonical_carpet_sudoku(
        &mut self,
        sudoku: DBCanonicalCarpetSudoku
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(canonical_carpet_sudokus).values(&sudoku).execute(&mut self.connection)
    }

    pub fn insert_canonical_carpet_game(
        &mut self,
        game: DBNewCanonicalCarpetGame
    ) -> Result<DBCanonicalCarpetGame, diesel::result::Error> {
        diesel
            ::insert_into(canonical_carpet_games)
            .values(&game)
            .get_result::<DBCanonicalCarpetGame>(&mut self.connection)
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

    pub fn insert_multiple_canonical_carpet_sudokus(
        &mut self,
        sudoku: Vec<DBCanonicalCarpetSudoku>
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(canonical_carpet_sudokus).values(&sudoku).execute(&mut self.connection)
    }

    pub fn insert_multiple_canonical_carpets(
        &mut self,
        carpets: Vec<DBCanonicalCarpet>
    ) -> Result<usize, diesel::result::Error> {
        let mut inserted_carpets = 0;

        for carpets_chunk in carpets.chunks(16348) {
            inserted_carpets += diesel
                ::insert_into(canonical_carpets)
                .values(carpets_chunk)
                .execute(&mut self.connection)?;
        }

        Ok(inserted_carpets)
    }

    pub fn insert_multiple_canonical_carpet_games(
        &mut self,
        games: Vec<DBNewCanonicalCarpetGame>
    ) -> Result<Vec<DBCanonicalCarpetGame>, diesel::result::Error> {
        diesel
            ::insert_into(canonical_carpet_games)
            .values(&games)
            .get_results::<DBCanonicalCarpetGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////   GEL RANDOM   /////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_random_canonical_sudokus(
        &mut self,
        n: u8
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        canonical_sudokus
            .filter(sudoku_n.eq(n as i16))
            .order(rand())
            .limit(1)
            .get_result::<DBCanonicalSudoku>(&mut self.connection)
            .map(SimpleSudoku::db_from_filled)
    }

    pub fn get_random_canonical_sudoku_game(
        &mut self,
        n: i16,
        difficulty: i16
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let game_info = canonical_sudoku_games
            .filter(sudoku_game_n.eq(n).and(sudoku_game_difficulty.eq(difficulty)))
            .order(rand())
            .limit(1)
            .get_result::<DBCanonicalSudokuGame>(&mut self.connection)?;

        canonical_sudokus
            .filter(filled_board_hash.eq(game_info.sudoku_game_filled_board_hash))
            .get_result::<DBCanonicalSudoku>(&mut self.connection)
            .map(|filled_info| SimpleSudoku::db_from_game(game_info, filled_info))
    }

    pub fn get_random_canonical_carpet(&mut self, n: i16, difficulty: i16, pattern: i16) {
        todo!()
    }
}
