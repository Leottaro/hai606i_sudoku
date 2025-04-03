use diesel::PgConnection;
use schema::*;

pub mod db;
pub mod schema;

pub struct Database {
    connection: PgConnection,
}

#[derive(Insertable, Selectable, Queryable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_sudokus)]
pub struct DBCanonicalSudoku {
    pub filled_board_hash: i64,
    pub sudoku_n: i16,
    pub canonical_board: Vec<u8>,
}

#[derive(Insertable, Selectable, Queryable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_sudoku_squares)]
pub struct DBCanonicalSudokuSquare {
    pub square_filled_board_hash: i64,
    pub square_id: i16,
    pub square_hash: i64,
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_sudoku_games)]
pub struct DBCanonicalSudokuGame {
    pub game_id: i32,
    pub game_filled_board_hash: i64,
    pub game_n: i16,
    pub game_board: Vec<u8>,
    pub game_difficulty: i16,
    pub game_filled_cells: i16,
}

#[derive(Insertable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_sudoku_games)]
pub struct DBNewCanonicalSudokuGame {
    pub game_filled_board_hash: i64,
    pub game_n: i16,
    pub game_board: Vec<u8>,
    pub game_difficulty: i16,
    pub game_filled_cells: i16,
}

impl From<DBCanonicalSudokuGame> for DBNewCanonicalSudokuGame {
    fn from(game: DBCanonicalSudokuGame) -> Self {
        Self {
            game_filled_board_hash: game.game_filled_board_hash,
            game_n: game.game_n,
            game_board: game.game_board,
            game_difficulty: game.game_difficulty,
            game_filled_cells: game.game_filled_cells,
        }
    }
}
