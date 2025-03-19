// @generated automatically by Diesel CLI.

diesel::table! {
    simple_sudoku_filled (filled_board_hash) {
        filled_board_hash -> Unsigned<Bigint>,
        filled_n -> Unsigned<Tinyint>,
        filled_board -> Tinyblob,
        filled_up_left_corner -> Unsigned<Bigint>,
        filled_up_right_corner -> Unsigned<Bigint>,
        filled_bottom_left_corner -> Unsigned<Bigint>,
        filled_bottom_right_corner -> Unsigned<Bigint>,
    }
}

diesel::table! {
    simple_sudoku_games (game_id) {
        game_id -> Integer,
        filled_board_hash -> Unsigned<Bigint>,
        game_n -> Unsigned<Tinyint>,
        game_board -> Tinyblob,
        game_difficulty -> Unsigned<Tinyint>,
    }
}

diesel::joinable!(simple_sudoku_games -> simple_sudoku_filled (filled_board_hash));

diesel::allow_tables_to_appear_in_same_query!(simple_sudoku_filled, simple_sudoku_games,);
