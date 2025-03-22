// @generated automatically by Diesel CLI.

diesel::table! {
    simple_sudoku_canonical (canonical_board_hash) {
        canonical_board_hash -> Unsigned<Bigint>,
        sudoku_n -> Unsigned<Tinyint>,
        canonical_board -> Tinyblob,
    }
}

diesel::table! {
    simple_sudoku_canonical_squares (canonical_board_hash, square_id) {
        canonical_board_hash -> Unsigned<Bigint>,
        square_id -> Unsigned<Tinyint>,
        square_hash -> Unsigned<Bigint>,
    }
}

diesel::table! {
    simple_sudoku_games (game_id) {
        game_id -> Integer,
        canonical_board_hash -> Unsigned<Bigint>,
        game_n -> Unsigned<Tinyint>,
        game_board -> Tinyblob,
        game_difficulty -> Unsigned<Tinyint>,
        game_filled_cells -> Unsigned<Tinyint>,
    }
}

diesel::joinable!(simple_sudoku_canonical_squares -> simple_sudoku_canonical (canonical_board_hash));
diesel::joinable!(simple_sudoku_games -> simple_sudoku_canonical (canonical_board_hash));

diesel::allow_tables_to_appear_in_same_query!(
    simple_sudoku_canonical,
    simple_sudoku_canonical_squares,
    simple_sudoku_games,
);
