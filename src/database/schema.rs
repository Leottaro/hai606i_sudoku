// @generated automatically by Diesel CLI.

diesel::table! {
    canonical_sudoku_games (game_id) {
        game_id -> Int4,
        game_filled_board_hash -> Int8,
        game_n -> Int2,
        game_board -> Bytea,
        game_difficulty -> Int2,
        game_filled_cells -> Int2,
    }
}

diesel::table! {
    canonical_sudoku_squares (square_filled_board_hash, square_id) {
        square_filled_board_hash -> Int8,
        square_id -> Int2,
        square_hash -> Int8,
    }
}

diesel::table! {
    canonical_sudokus (filled_board_hash) {
        filled_board_hash -> Int8,
        sudoku_n -> Int2,
        canonical_board -> Bytea,
    }
}

diesel::joinable!(canonical_sudoku_games -> canonical_sudokus (game_filled_board_hash));
diesel::joinable!(canonical_sudoku_squares -> canonical_sudokus (square_filled_board_hash));

diesel::allow_tables_to_appear_in_same_query!(
    canonical_sudoku_games,
    canonical_sudoku_squares,
    canonical_sudokus,
);
