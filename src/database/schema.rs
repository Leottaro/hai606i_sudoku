// @generated automatically by Diesel CLI.

diesel::table! {
    canonical_carpet_games (carpet_game_id) {
        carpet_game_id -> Int4,
        carpet_game_carpet_filled_board_hash -> Int8,
        carpet_game_difficulty -> Int2,
        carpet_game_filled_cells -> Bytea,
        carpet_game_filled_cells_count -> Int2,
    }
}

diesel::table! {
    canonical_carpet_sudokus (carpet_sudoku_carpet_filled_board_hash, carpet_sudoku_i, carpet_sudoku_filled_board_hash) {
        carpet_sudoku_carpet_filled_board_hash -> Int8,
        carpet_sudoku_i -> Int2,
        carpet_sudoku_filled_board_hash -> Int8,
    }
}

diesel::table! {
    canonical_carpets (carpet_filled_board_hash) {
        carpet_filled_board_hash -> Int8,
        carpet_n -> Int2,
        carpet_sudoku_number -> Int2,
        carpet_pattern -> Int2,
        carpet_pattern_size -> Nullable<Int2>,
    }
}

diesel::table! {
    canonical_sudoku_games (sudoku_game_id) {
        sudoku_game_id -> Int4,
        sudoku_game_filled_board_hash -> Int8,
        sudoku_game_difficulty -> Int2,
        sudoku_game_filled_cells -> Bytea,
        sudoku_game_filled_cells_count -> Int2,
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

diesel::joinable!(canonical_carpet_games -> canonical_carpets (carpet_game_carpet_filled_board_hash));
diesel::joinable!(canonical_carpet_sudokus -> canonical_carpets (carpet_sudoku_carpet_filled_board_hash));
diesel::joinable!(canonical_carpet_sudokus -> canonical_sudokus (carpet_sudoku_filled_board_hash));
diesel::joinable!(canonical_sudoku_games -> canonical_sudokus (sudoku_game_filled_board_hash));
diesel::joinable!(canonical_sudoku_squares -> canonical_sudokus (square_filled_board_hash));

diesel::allow_tables_to_appear_in_same_query!(
    canonical_carpet_games,
    canonical_carpet_sudokus,
    canonical_carpets,
    canonical_sudoku_games,
    canonical_sudoku_squares,
    canonical_sudokus,
);
