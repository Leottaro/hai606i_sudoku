use diesel::MysqlConnection;
use schema::{simple_sudoku_canonical, simple_sudoku_canonical_squares, simple_sudoku_games};

pub mod db;
pub mod schema;

pub struct Database {
    connection: MysqlConnection,
}

#[derive(Insertable, Selectable, Queryable, Clone)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = simple_sudoku_canonical)]
pub struct DBSimpleSudokuCanonical {
    pub canonical_board_hash: u64,
    pub sudoku_n: u8,
    pub canonical_board: Vec<u8>,
}

#[derive(Insertable, Selectable, Queryable, Clone)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = simple_sudoku_canonical_squares)]
pub struct DBSimpleSudokuCanonicalSquares {
    pub square_canonical_board_hash: u64,
    pub square_id: u8,
    pub square_hash: u64,
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = simple_sudoku_games)]
pub struct DBSimpleSudokuGame {
    pub game_id: i32,
    pub game_canonical_board_hash: u64,
    pub game_n: u8,
    pub game_board: Vec<u8>,
    pub game_difficulty: u8,
    pub game_filled_cells: u16,
}

#[derive(Insertable, Clone)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = simple_sudoku_games)]
pub struct DBNewSimpleSudokuGame {
    pub game_canonical_board_hash: u64,
    pub game_n: u8,
    pub game_board: Vec<u8>,
    pub game_difficulty: u8,
    pub game_filled_cells: u16,
}

impl From<DBSimpleSudokuGame> for DBNewSimpleSudokuGame {
    fn from(game: DBSimpleSudokuGame) -> Self {
        Self {
            game_canonical_board_hash: game.game_canonical_board_hash,
            game_n: game.game_n,
            game_board: game.game_board,
            game_difficulty: game.game_difficulty,
            game_filled_cells: game.game_filled_cells,
        }
    }
}
