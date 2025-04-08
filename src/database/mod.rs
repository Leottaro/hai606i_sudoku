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
    pub sudoku_game_id: i32,
    pub sudoku_game_filled_board_hash: i64,
    pub sudoku_game_n: i16,
    pub sudoku_game_difficulty: i16,
    pub sudoku_game_filled_cells: Vec<u8>,
    pub sudoku_game_filled_cells_count: i16,
}

#[derive(Insertable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_sudoku_games)]
pub struct DBNewCanonicalSudokuGame {
    pub sudoku_game_filled_board_hash: i64,
    pub sudoku_game_n: i16,
    pub sudoku_game_difficulty: i16,
    pub sudoku_game_filled_cells: Vec<u8>,
    pub sudoku_game_filled_cells_count: i16,
}

impl From<DBCanonicalSudokuGame> for DBNewCanonicalSudokuGame {
    fn from(game: DBCanonicalSudokuGame) -> Self {
        DBNewCanonicalSudokuGame {
            sudoku_game_filled_board_hash: game.sudoku_game_filled_board_hash,
            sudoku_game_n: game.sudoku_game_n,
            sudoku_game_difficulty: game.sudoku_game_difficulty,
            sudoku_game_filled_cells: game.sudoku_game_filled_cells,
            sudoku_game_filled_cells_count: game.sudoku_game_filled_cells_count,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_carpets)]
pub struct DBCanonicalCarpet {
    pub carpet_filled_board_hash: i64,
    pub carpet_n: i16,
    pub carpet_sudoku_number: i16,
    pub carpet_pattern: i16,
    pub carpet_pattern_size: Option<i16>,
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_carpet_sudokus)]
pub struct DBCanonicalCarpetSudoku {
    pub carpet_sudoku_carpet_filled_board_hash: i64,
    pub carpet_sudoku_i: i16,
    pub carpet_sudoku_filled_board_hash: i64,
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_carpet_games)]
pub struct DBCanonicalCarpetGame {
    pub carpet_game_id: i32,
    pub carpet_game_carpet_filled_board_hash: i64,
    pub carpet_game_n: i16,
    pub carpet_game_difficulty: i16,
    pub carpet_game_filled_cells: Vec<u8>,
    pub carpet_game_filled_cells_count: i16,
}

#[derive(Insertable, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = canonical_carpet_games)]
pub struct DBNewCanonicalCarpetGame {
    pub carpet_game_carpet_filled_board_hash: i64,
    pub carpet_game_n: i16,
    pub carpet_game_difficulty: i16,
    pub carpet_game_filled_cells: Vec<u8>,
    pub carpet_game_filled_cells_count: i16,
}
