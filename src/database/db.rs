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

    pub fn insert_multiple_simple_sudoku_canonical(
        &mut self,
        sudokus: Vec<DBSimpleSudokuCanonical>,
        squares: Vec<DBSimpleSudokuCanonicalSquares>,
    ) -> Result<usize, diesel::result::Error> {
        let k1 = diesel::insert_or_ignore_into(simple_sudoku_canonical)
            .values(sudokus)
            .execute(&mut self.connection)?;

        let k2 = diesel::insert_or_ignore_into(simple_sudoku_canonical_squares)
            .values(squares)
            .execute(&mut self.connection)?;

        Ok(k1 + k2)
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

    pub fn get_random_simple_sudoku_canonical(
        &mut self,
        n: usize,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let nb_max = simple_sudoku_canonical
            .filter(sudoku_n.eq(n as u8))
            .count()
            .get_result::<i64>(&mut self.connection)?;
        simple_sudoku_canonical
            .filter(sudoku_n.eq(n as u8))
            .order(rand())
            .limit(nb_max - 1)
            .get_result::<DBSimpleSudokuCanonical>(&mut self.connection)
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

impl DBSimpleSudokuCanonical {
    pub fn to_sudoku(&self) -> SimpleSudoku {
        let mut sudoku = SimpleSudoku::new(self.sudoku_n as usize);
        for y in 0..sudoku.get_n2() {
            for x in 0..sudoku.get_n2() {
                let value = self.canonical_board
                    [y * self.sudoku_n as usize * self.sudoku_n as usize + x]
                    as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value).unwrap();
                }
            }
        }
        sudoku
    }
}
impl DBSimpleSudokuGame {
    pub fn to_sudoku(&self) -> SimpleSudoku {
        let mut sudoku = SimpleSudoku::new(self.game_n as usize);
        for y in 0..sudoku.get_n2() {
            for x in 0..sudoku.get_n2() {
                let value =
                    self.game_board[y * self.game_n as usize * self.game_n as usize + x] as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value).unwrap();
                }
            }
        }
        sudoku
    }
}

impl DBNewSimpleSudokuGame {
    pub fn to_sudoku(&self) -> SimpleSudoku {
        let mut sudoku = SimpleSudoku::new(self.game_n as usize);
        for y in 0..sudoku.get_n2() {
            for x in 0..sudoku.get_n2() {
                let value =
                    self.game_board[y * self.game_n as usize * self.game_n as usize + x] as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value).unwrap();
                }
            }
        }
        sudoku
    }
}
